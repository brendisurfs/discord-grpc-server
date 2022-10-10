use std::path::Path;

use reqwest::{
    header::{ACCEPT, CONNECTION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::{
    http::Http,
    model::{prelude::AttachmentType, webhook::Webhook},
};
use std::error::Error;
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug)]
struct ReturnJson {
    event: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DreamPost<'a> {
    pub prompt: String,
    pub iterations: usize,
    pub steps: usize,
    pub cfg_scale: f32,
    pub sampler_name: &'a str,
    pub width: usize,
    pub height: usize,
    pub seed: i32,
    pub variation_amount: f32,
    pub with_variations: &'a str,
    pub initimg: Option<&'a str>,
    pub strength: f32,
    pub fit: &'a str,
    pub gfpgan_strength: f32,
    pub upscale_level: &'a str,
    pub upscale_strength: f32,
    pub initimg_name: &'a str,
}

impl<'a> DreamPost<'a> {
    pub fn new(prompt: String) -> Self {
        DreamPost {
            prompt,
            iterations: 1,
            steps: 50,
            cfg_scale: 7.5,
            sampler_name: "plms",
            width: 640,
            height: 640,
            seed: -1,
            variation_amount: 0.11f32,
            with_variations: "",
            initimg: None,
            initimg_name: "",
            strength: 0.75,
            fit: "on",
            gfpgan_strength: 0.8,
            upscale_level: "",
            upscale_strength: 0.75,
        }
    }

    /// `send_prompt`
    pub async fn send_prompt(&self) -> Result<(), Box<dyn Error>> {
        dotenv::dotenv().ok();
        let ngrok_url =
            std::env::var("NGROK_URL").expect("could not get ngrok url. Is it set?");

        let return_prompt = &self.prompt.clone();

        let post_prompt = json!(&self).to_string();

        let client = Client::new();
        let res = client
            .post(ngrok_url)
            .header(CONTENT_TYPE, "application/json")
            .header(CONNECTION, "keep-alive")
            .header(ACCEPT, "*")
            .body(post_prompt);

        match res.send().await {
            Ok(res) => {
                if res.status() == 200 {
                    let status = res.text().await.expect("could not get text");
                    let base64_from_server = status
                        .lines()
                        .skip_while(|line| line.contains("event"))
                        .collect::<String>();
                    println!("received image");
                    let decoded_png = base64::decode(base64_from_server)
                        .expect("could not decode string from server");

                    // create an output temp image to store.
                    // this will get written over later.
                    let mut file_to_send = File::create("output/tmp.png")
                        .await
                        .expect("could not create temp file");
                    file_to_send
                        .write_all(&decoded_png)
                        .await
                        .expect("could not write to file");

                    send_webhook(return_prompt.to_string()).await;
                } else {
                    ()
                }
            }
            Err(why) => {
                eprintln!("why: {why:?}");
            }
        }
        Ok(())
    }
}

async fn send_webhook(return_prompt: String) {
    let http = Http::new("");
    let webhook_api =
        std::env::var("DISCORD_WEBHOOK").expect("webhook_api needs to be set");
    let attachment = AttachmentType::Path(Path::new("output/tmp.png"));
    let wh = Webhook::from_url(&http, &webhook_api)
        .await
        .expect("could not make webhook");

    wh.execute(&http, false, |w| {
        w.content(return_prompt)
            .username("Sacred Telemetry")
            .add_file(attachment)
    })
    .await
    .unwrap();
}

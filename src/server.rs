use std::path::Path;

use serde::Deserialize;
use serde_json::value::Serializer;
use serenity::{
    http::Http,
    model::{prelude::AttachmentType, webhook::Webhook},
};
use tracing::log::info;
use zeromq::{Socket, SocketRecv};

#[derive(Deserialize, Debug)]
pub struct PromptMsg {
    pub user_name: String,
    pub prompt: String,
}

// the zeromq server to hold
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let server_addr = "tcp://127.0.0.1:5560";

    let mut mq_server = zeromq::RepSocket::new();
    mq_server
        .connect(&server_addr)
        .await
        .expect("failed to connect server");

    info!("server started on 5560");

    loop {
        let repl: String = mq_server.recv().await?.try_into()?;
        let serialized_repl = serde_json::from_str::<PromptMsg>(&repl).unwrap();
        info!("received msg: {serialized_repl:?}");

        build_and_send_webhook(serialized_repl.prompt).await;
    }
}

// builds a quick webhook handler and sends the webhook.
async fn build_and_send_webhook(prompt: String) {
    let http = Http::new("");
    let webhook_api = std::env::var("DISCORD_WEBHOOK").unwrap();
    let attachment = AttachmentType::Path(Path::new("resources/imcrying.jpg"));
    let webhook = Webhook::from_url(&http, &webhook_api)
        .await
        .expect("could not build webhook");

    let response_msg = format!("user prompt: {prompt}");

    webhook
        .execute(&http, false, |w| {
            w.content(response_msg)
                .username("Webhook test")
                .add_file(attachment)
        })
        .await
        .expect("could not execute webook");
}

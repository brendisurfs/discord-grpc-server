mod common;
use common::requestor::{self, DreamPost};
use serde::Deserialize;
use serenity::{
    http::Http,
    model::{
        prelude::{AttachmentType, MessageFlags, UserId},
        webhook::Webhook,
    },
    utils::MessageBuilder,
};
use std::{path::Path, time::Duration};
use tokio::time::sleep;
use tonic::codegen::http::uri::Port;
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
        // simulate the processing time.
        sleep(Duration::from_secs(10)).await;
        // send the final image back to the server.
        build_and_send_webhook(serialized_repl).await;
    }
}

// builds a quick webhook handler and sends the webhook.
async fn build_and_send_webhook(prompt_msg: PromptMsg) {
    let PromptMsg { user_name, prompt } = prompt_msg;
    send_prompt_to_generator(prompt.clone())
        .await
        .expect("could not send prompt to generator");
}

async fn send_prompt_to_generator(
    prompt: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: these should have a default later;
    let struct_prompt = DreamPost::new(prompt);
    struct_prompt
        .send_prompt()
        .await
        .expect("could not send prompt");
    Ok(())
}

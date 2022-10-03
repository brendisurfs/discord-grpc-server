use std::{
    borrow::{Borrow, Cow},
    path::Path,
};

use serenity::{
    http::Http,
    model::{
        prelude::{AttachmentType, MessageFlags},
        webhook::Webhook,
    },
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

// the zeromq server to hold
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    build_and_send_webhook().await;
}

// builds a quick webhook handler and sends the webhook.
async fn build_and_send_webhook() {
    let webhook_api = std::env::var("DISCORD_WEBHOOK").unwrap();
    let http = Http::new("");

    let attachment = AttachmentType::Path(Path::new("resources/imcrying.jpg"));

    let webhook = Webhook::from_url(&http, &webhook_api)
        .await
        .expect("could not build webhook");
    webhook
        .execute(&http, false, |w| {
            w.content("hello there")
                .username("Webhook test")
                .add_file(attachment)
        })
        .await
        .expect("could not execute webook");
}

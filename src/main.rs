use std::collections::VecDeque;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::Msg;
use tokio::sync::mpsc;
mod prompt {
	include!("prompt.rs");
}

// shared data between green threads to see who is in the queue next.
#[derive(Debug)]
struct SharedQueue {
	prompts: VecDeque<mpsc::Sender<Msg>>,
}

impl SharedQueue {
	fn new() -> Self {
		SharedQueue {
			prompts: VecDeque::new(),
		}
	}

	async fn broadcast(&self, msg: Msg) {
		for tx in &self.prompts {
			match tx.send(msg.clone()).await {
				Ok(_) => {}
				Err(_) => {
					println!(
						"send error: to {}, {:?}",
						msg.user_name, msg.prompt
					)
				}
			}
		}
	}
}

#[tokio::main]
async fn main() {
	println!("Hello, world!");
}

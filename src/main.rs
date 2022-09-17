use std::collections::VecDeque;
use std::sync::Arc;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{Msg, ReturnPrompt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tonic::{Request, Response, Status};
mod prompt {
	include!("prompt.rs");
}

// shared data between green threads to see who is in the queue next.
#[derive(Debug)]
struct SharedQueue {
	prompts: VecDeque<Msg>,
}

impl SharedQueue {
	fn new() -> Self {
		SharedQueue {
			prompts: VecDeque::new(),
		}
	}

	// async fn broadcast(&self, msg: Msg) {
	// 	for tx in &self.prompts {
	// 		match tx.send(msg.clone()).await {
	// 			Ok(_) => {}
	// 			Err(_) => {
	// 				println!(
	// 					"send error: to {}, {:?}",
	// 					msg.user_name, msg.prompt
	// 				)
	// 			}
	// 		}
	// 	}
	// }
}

#[derive(Debug)]
struct PromptService {
	shared_queue: Arc<Mutex<SharedQueue>>,
}

impl PromptService {
	/// creates a new prompt service to handle incoming prompts.
	fn new(shared_queue: Arc<Mutex<SharedQueue>>) -> Self {
		PromptService { shared_queue }
	}
}

#[tonic::async_trait]
impl PromptReq for PromptService {
	/// `async fn send_prompt` will handle sending back info to the discord bot.
	///
	async fn send_prompt(
		&self,
		request: Request<Msg>,
	) -> Result<Response<ReturnPrompt>, Status> {
		let inner_data = request.into_inner();
		let user_name = inner_data.user_name;
		let user_prompt = inner_data.prompt;
		// -----ALL THIS DOWN HERE COMES AFTER THE BLOCKING PROCESS FINISHES.---
		let return_image: Vec<u8> = vec![];

		// send this back to the discord bot.
		let cloned_uname = user_name.clone();
		let response_obj = ReturnPrompt {
			user_name: cloned_uname,
			jpg: return_image,
		};

		let msg = Msg {
			user_name,
			prompt: user_prompt,
		};

		let mut queue = self
			.shared_queue
			.try_lock()
			.expect("could not lock queue mutex");

		queue.prompts.push_back(msg);

		Ok(Response::new(response_obj))
	}
}

#[tokio::main]
async fn main() {
	println!("Hello, world!");
}

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{Empty, Msg, ReturnPrompt};
use tokio::sync::Mutex;
use tonic::codegen::futures_core::Stream;
use tonic::transport::Server;
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

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<ReturnPrompt, Status>> + Send>>;
type ServerStreamingEchoStream = ResponseStream;

#[tonic::async_trait]
impl PromptReq for PromptService {
    /// `async fn send_prompt` will handle sending back info to the discord bot.
    ///
    async fn send_prompt(&self, request: Request<Msg>) -> EchoResult<ReturnPrompt> {
        let inner_data = request.into_inner();
        let user_name = inner_data.user_name;
        let user_prompt = inner_data.prompt;
        // -----ALL THIS DOWN HERE COMES AFTER THE BLOCKING PROCESS FINISHES.---
        // let return_image: Vec<u8> = vec![];

        let response_object = ReturnPrompt {
            user_name: user_name.clone(),
            jpg: user_prompt.clone(),
        };

        let repeater = std::iter::repeat(response_object);

        let msg = Msg {
            user_name,
            prompt: user_prompt,
        };

        let mut queue = self
            .shared_queue
            .try_lock()
            .expect("could not lock queue mutex");

        // handle if the queue is too large.
        if queue.prompts.len() > 100 {
            Err(Status::new(
                tonic::Code::Unavailable,
                "queue is full, please try again later",
            ))
        } else {
            queue.prompts.push_back(msg);

            Ok(Response::new(response_object))
        }
    }

    // async fn process_prompt(&self, request: Request<Empty>) -> EchoResult<ReturnPrompt> {
    //     println!("not implemented");
    //     let inner_data = request.into_inner();
    //     let user_name = inner_data.user_name;
    //     let user_prompt = inner_data.prompt;
    //
    //     let response_object = ReturnPrompt {
    //         user_name: user_name.clone(),
    //         jpg: user_prompt.clone(),
    //     };
    //     Ok()
    // }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051"
        .parse::<SocketAddr>()
        .expect("could not parse addr");

    println!("prompt server listening on: {}", addr);

    let shared_queue = Arc::new(Mutex::new(SharedQueue::new()));
    let prompt_service = PromptService::new(shared_queue.clone());

    Server::builder()
        .add_service(PromptReqServer::new(prompt_service))
        .serve(addr)
        .await?;

    Ok(())
}

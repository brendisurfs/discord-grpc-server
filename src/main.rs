use std::collections::VecDeque;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{PromptRequest, PromptResponse};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::codegen::futures_core::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
mod prompt {
    include!("prompt.rs");
}

// shared data between green threads to see who is in the queue next.
#[derive(Debug)]
struct SharedQueue {
    prompts: VecDeque<PromptRequest>,
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

type PromptResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<PromptResponse, Status>> + Send>>;

#[tonic::async_trait]
impl PromptReq for PromptService {
    /// `async fn send_prompt` will handle sending back info to the discord bot.

    // implement SendPromptStream
    type SendPromptStream = ResponseStream;

    async fn send_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> PromptResult<Self::SendPromptStream> {
        let inner_data = request.into_inner();
        let user_name = inner_data.user_name;
        let user_prompt = inner_data.prompt;
        // -----ALL THIS DOWN HERE COMES AFTER THE BLOCKING PROCESS FINISHES.---
        // let return_image: Vec<u8> = vec![];

        let response_object = PromptResponse {
            user_name: user_name.clone(),
            jpg: user_prompt.clone(),
        };

        // create our iterator to loop over the stream.
        let repeater = std::iter::repeat(response_object);

        // throttle repeater and incoming streams.
        let throttle_duration = Duration::from_secs(1);
        let mut stream =
            Box::pin(tokio_stream::iter(repeater).throttle(throttle_duration));

        // little bit larger buffer size, but lets us know when something
        // inside the loop works or doesnt.
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                let send_item_result = tx.send(Result::<_, Status>::Ok(item)).await;
                match send_item_result {
                    Ok(_) => {
                        // item (server response) was queued to send to the client.
                    }
                    Err(why) => {
                        // output_stream built from rx is dropped.
                        eprintln!("received an error: {:?}", why);
                        break;
                    }
                }
            }
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);

        Ok(Response::new(Box::pin(output_stream) as ResponseStream))
    }
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

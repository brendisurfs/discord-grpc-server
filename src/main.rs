mod queue;
use std::borrow::Borrow;
use std::fs;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{PromptRequest, PromptResponse};
use queue::Queue;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::codegen::futures_core::Stream;
use tonic::codegen::CompressionEncoding;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
mod prompt {
    include!("prompt.rs");
}
#[derive(Debug)]
struct PromptService {
    queue: Arc<Mutex<Queue<PromptRequest>>>,
}

impl PromptService {
    /// creates a new prompt service to handle incoming prompts.
    fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Queue::new())),
        }
    }
}

type PromptResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<PromptResponse, Status>> + Send>>;

#[tonic::async_trait]
impl PromptReq for PromptService {
    // implement SendPromptStream
    type SendPromptStream = ResponseStream;

    // handle on send prompt.
    async fn send_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> PromptResult<Self::SendPromptStream> {
        println!("got prompt: {:#?}", request.remote_addr());

        let inner_data = request.into_inner();
        let user_name = &inner_data.user_name;
        // let user_prompt = &inner_data.prompt;

        self.queue
            .lock()
            .expect("could not lock thread")
            .enqueue(inner_data.clone());

        println!("{:?}", self.queue.clone());

        let mut test_file = tokio::fs::File::open("resources/imcrying.jpg")
            .await
            .expect("could not open file");

        let mut buffer_vec = Vec::new();
        let _ = test_file.read_to_end(&mut buffer_vec).await;

        let response_jpg = base64::encode(buffer_vec);
        // artificial test of processing
        tokio::time::sleep(Duration::from_secs(2)).await;

        let response_object = PromptResponse {
            user_name: user_name.clone(),
            jpg: response_jpg,
        };

        let repeater = std::iter::repeat(response_object);
        // create our iterator to loop over the stream.
        // let iter_queue = self.queue.into_inner().unwrap().into_iter();

        // throttle repeater and incoming streams.
        let throttle_duration = Duration::from_secs(1);
        let mut stream =
            Box::pin(tokio_stream::iter(repeater).throttle(throttle_duration));

        // little bit larger buffer size, but lets us know when something
        // inside the loop works or doesnt.
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                // NOTE: THIS IS WHERE WE NEED TO RUN OUT BLOCKING GENERATOR.
                let send_item_result = tx.send(Result::<_, Status>::Ok(item)).await;
                match send_item_result {
                    Ok(_) => (),
                    Err(_) => {
                        // output_stream built from rx is dropped.
                        eprintln!("rx and output_stream timed out");
                        break;
                    }
                }
            }
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

    let prompt_service = PromptService::new();

    let service =
        PromptReqServer::new(prompt_service).send_compressed(CompressionEncoding::Gzip);

    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}

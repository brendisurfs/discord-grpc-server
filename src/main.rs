mod queue;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{PromptRequest, PromptResponse};
use tokio::io::AsyncReadExt;
use tonic::codegen::CompressionEncoding;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
mod prompt {
    include!("prompt.rs");
}
#[derive(Debug)]
struct PromptService {
    queue: Arc<Mutex<Vec<PromptRequest>>>,
}

impl PromptService {
    /// creates a new prompt service to handle incoming prompts.
    fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// type PromptResult<T> = Result<Response<T>, Status>;
// type ResponseStream = Pin<Box<dyn Stream<Item = Result<PromptResponse, Status>> + Send>>;

#[tonic::async_trait]
impl PromptReq for PromptService {
    // type SendPromptStream = ResponseStream;

    async fn send_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> Result<Response<PromptResponse>, Status> {
        println!("got prompt: {:#?}", request.remote_addr());

        let inner_data = request.into_inner();
        let user_name = &inner_data.user_name;

        self.queue
            .lock()
            .expect("could not lock thread")
            .push(inner_data.clone());

        let mut test_file = tokio::fs::File::open("resources/imcrying.jpg")
            .await
            .expect("could not open file");

        let mut vec_buffer = Vec::new();
        let _ = test_file.read_to_end(&mut vec_buffer).await;

        let response_object = PromptResponse {
            user_name: user_name.clone(),
            jpg: vec_buffer,
        };

        Ok(Response::new(response_object))
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

    let server = Server::builder().add_service(service);
    server
        .serve(addr)
        .await
        .expect("could not serve grpc server");

    Ok(())
}

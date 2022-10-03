use std::net::SocketAddr;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{Empty, PromptRequest, PromptResponse};
use tokio::io::AsyncReadExt;
use tonic::codegen::CompressionEncoding;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
mod prompt {
    include!("prompt.rs");
}
#[derive(Debug)]
struct PromptService;

#[tonic::async_trait]
impl PromptReq for PromptService {
    // type SendPromptStream = ResponseStream;
    fn receive_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }

    async fn send_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> Result<Response<PromptResponse>, Status> {
        println!("got prompt: {:#?}", request.remote_addr());

        let inner_data = request.into_inner();
        let user_name = &inner_data.user_name;

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

    let service =
        PromptReqServer::new(PromptService).send_compressed(CompressionEncoding::Gzip);

    let server = Server::builder().add_service(service);
    server
        .serve(addr)
        .await
        .expect("could not serve grpc server");

    Ok(())
}

use std::net::SocketAddr;

use prompt::prompt_req_server::{PromptReq, PromptReqServer};
use prompt::{Empty, PromptRequest, PromptResponse};
use serde_json::json;
use tonic::codegen::CompressionEncoding;
use tonic::transport::Server;
use tonic::{async_trait, Request, Response, Status};
use zeromq::{ReqSocket, Socket, SocketSend};

mod prompt {
    include!("prompt.rs");
}
#[derive(Debug)]
struct PromptService;

impl PromptService {
    async fn run_mq_client(
        &self,
        msg: PromptRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut socket = zeromq::ReqSocket::new();
        socket.connect("tcp://127.0.0.1:5559").await?;

        tokio::spawn(async move {
            let PromptRequest { user_name, prompt } = msg;
            let msg = json!({
                "user_name": user_name,
                "prompt": prompt,
            })
            .to_string();

            socket.send(msg.into()).await.expect("could not send to mq");
        });
        Ok(())
    }
}

#[async_trait]
impl PromptReq for PromptService {
    async fn receive_prompt(
        &self,
        request: Request<PromptRequest>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        // send the request to msg handler.
        self.run_mq_client(req)
            .await
            .expect("error with run mq client");
        Ok(Response::new(Empty {}))
    }
    async fn send_prompt(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<PromptResponse>, Status> {
        let msg = PromptResponse {
            user_name: "Brendi".into(),
            jpg: vec![0; 1024].into(),
        };

        Ok(Response::new(msg))
    }
}

// #[tonic::async_trait]
// impl PromptReq for PromptService {
//     // type SendPromptStream = ResponseStream;
//     fn receive_prompt(
//         &self,
//         request: Request<PromptRequest>,
//     ) -> Result<Response<Empty>, Status> {
//         Ok(Response::new(Empty {}))
//     }
//
//     async fn send_prompt(
//         &self,
//         request: Request<PromptRequest>,
//     ) -> Result<Response<PromptResponse>, Status> {
//         println!("got prompt: {:#?}", request.remote_addr());
//
//         let inner_data = request.into_inner();
//         let user_name = &inner_data.user_name;
//
//         let mut test_file = tokio::fs::File::open("resources/imcrying.jpg")
//             .await
//             .expect("could not open file");
//
//         let mut vec_buffer = Vec::new();
//         let _ = test_file.read_to_end(&mut vec_buffer).await;
//
//         let response_object = PromptResponse {
//             user_name: user_name.clone(),
//             jpg: vec_buffer,
//         };
//
//         Ok(Response::new(response_object))
//     }
// }

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

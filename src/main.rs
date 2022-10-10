mod common;
use prompt::prompt_req_server::PromptReq;
use prompt::{Empty, PromptRequest, PromptResponse};
use serde_json::json;
use tonic::{async_trait, Request, Response, Status};
use zeromq::{Socket, SocketSend};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

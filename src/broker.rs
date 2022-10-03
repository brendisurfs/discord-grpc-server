use std::error::Error;
use zeromq::Socket;

// zeromq broker to balance the msgs.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut frontend = zeromq::RouterSocket::new();
    frontend.bind("tcp://127.0.0.1:5559").await?;

    let mut backend = zeromq::DealerSocket::new();
    backend.bind("tcp://127.0.0.1:5560").await?;

    let mut capture = zeromq::PubSocket::new();
    capture.bind("tcp://127.0.0.1:9999").await?;

    zeromq::proxy(frontend, backend, Some(Box::new(capture))).await?;
    Ok(())
}

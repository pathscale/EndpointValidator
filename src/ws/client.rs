use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    seq: u32,
}

#[derive(Serialize)]
struct WsRequest<T: Serialize> {
    method: u32,
    seq: u32,
    params: T,
}

impl WsClient {
    pub async fn new(connect_addr: &str, auth_username: &str, auth_password: &str) -> Result<Self> {
        let mut req = <&str as IntoClientRequest>::into_client_request(connect_addr)
            .context("Failed to create client request")?;

        let protocol_header = format!(
            "0login, 1{}, 2{}, 3User, 424787297130491616, 5android",
            auth_username, auth_password
        );

        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str(&protocol_header).context("Invalid header value")?,
        );

        let (ws_stream, response) = connect_async(req)
            .await
            .context("Failed to connect to endpoint")?;

        println!("Server response headers:");
        for (name, value) in response.headers() {
            println!("  {}: {}", name, value.to_str().unwrap_or("invalid"));
        }

        let mut client = Self {
            stream: ws_stream,
            seq: 0,
        };

        let response = client.recv_raw().await?;
        println!("Initial response: {}", response);

        Ok(client)
    }

    pub async fn send_req(&mut self, method: u32, params: impl Serialize) -> Result<()> {
        self.seq += 1;
        let req = WsRequest {
            method,
            seq: self.seq,
            params,
        };

        let req_str = serde_json::to_string(&req).context("Failed to serialize request")?;

        self.stream
            .send(Message::Text(req_str))
            .await
            .context("Failed to send request")?;

        Ok(())
    }

    pub async fn recv_raw(&mut self) -> Result<serde_json::Value> {
        loop {
            let msg = self
                .stream
                .next()
                .await
                .ok_or_else(|| anyhow!("Connection closed"))?
                .context("Failed to receive message")?;

            match msg {
                Message::Text(text) => {
                    println!("Received raw message: {}", text);
                    return serde_json::from_str(&text)
                        .context("Failed to parse received message as JSON");
                }
                Message::Close(frame) => {
                    return Err(anyhow!("Server closed connection: {:?}", frame));
                }
                _ => {
                    println!("Ignoring non-text message");
                }
            }
        }
    }

    pub async fn close(mut self) -> Result<()> {
        self.stream
            .close(None)
            .await
            .context("Failed to close connection")?;
        Ok(())
    }
}

use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

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
    pub async fn new(connect_addr: &str, header: &str) -> Result<Self> {
        let mut req = <&str as IntoClientRequest>::into_client_request(connect_addr)
            .context("Failed to create client request")?;
        
        req.headers_mut()
            .insert("Sec-WebSocket-Protocol", HeaderValue::from_str(header)
            .context("Invalid header value")?);

        let (ws_stream, _) = connect_async(req).await.context("Failed to connect to endpoint")?;
        Ok(Self {
            stream: ws_stream,
            seq: 0,
        })
    }

    pub async fn send_req(&mut self, method: u32, params: impl Serialize) -> Result<()> {
        self.seq += 1;
        let req = serde_json::to_string(&WsRequest{
            method: method,
            seq: self.seq,
            params: params,
        })
        .context("Failed to serialize request")?;
        self.stream.send(Message::Text(req)).await.context("Failed to send request")?;
        Ok(())
    }

    pub async fn recv_raw(&mut self) -> Result<serde_json::Value> {
        // Get the next message from the stream, or return an error if the connection is closed
        let msg = self
            .stream
            .next()
            .await
            .ok_or_else(|| anyhow!("Connection closed"))?
            .context("Failed to receive message")?;
    
        let json_value = match msg {
            Message::Text(text) => {
                serde_json::from_str(&text).context("Failed to parse received message as JSON")?
            }
            _ => return Err(anyhow!("Received unexpected non-text message")),
        };
    
        Ok(json_value)
    }
    
    pub async fn close(mut self) -> Result<()> {
        self.stream.close(None).await.context("Failed to close connection")?;
        Ok(())
    }
}

use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use tracing::*;

// use crate::log::LogLevel;
use crate::ws::{WsRequestGeneric, WsResponseValue};

pub trait WsRequest: Serialize + DeserializeOwned + Send + Sync + Clone {
    type Response: WsResponse;
    const METHOD_ID: u32;
    const SCHEMA: &'static str;
}

pub trait WsResponse: Serialize + DeserializeOwned + Send + Sync + Clone {
    type Request: WsRequest;
}

pub struct WsClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    seq: u32,
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
        let req = serde_json::to_string(&WsRequestGeneric {
            method,
            seq: self.seq,
            params,
        })
        .context("Failed to serialize request")?;
        
        debug!("send req: {}", req);
        self.stream.send(Message::Text(req)).await.context("Failed to send request")?;
        Ok(())
    }

    pub async fn recv_raw(&mut self) -> Result<WsResponseValue> {
        let msg = self.stream.next().await.ok_or_else(|| anyhow!("Connection closed"))?
            .context("Failed to receive message")?;
        let resp: WsResponseValue = serde_json::from_str(&msg.to_string())
            .context("Failed to deserialize raw response")?;
        Ok(resp)
    }

    pub async fn close(mut self) -> Result<()> {
        self.stream.close(None).await.context("Failed to close connection")?;
        Ok(())
    }
}

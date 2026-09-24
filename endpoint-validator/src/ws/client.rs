//! The connection, on endpoint-libs' own client.
//!
//! endpoint-libs' `WsClient` runs on a nagoya reactor that the caller owns, so
//! every function here takes the reactor's [`Handle`]. There is no ambient
//! runtime: a socket opened on a reactor nobody polls never completes.

use std::time::Duration;

pub use endpoint_libs::libs::ws::WsClient;
use endpoint_libs::libs::ws::WsClientBuilder;
use eyre::{Result, WrapErr, bail};
use futures::FutureExt;
use futures::future::{Either, select};
pub use nagoya::reactor::Handle;

/// How long to wait for one frame before calling the service unresponsive.
pub const RECV_TIMEOUT: Duration = Duration::from_secs(15);

/// Open a connection, sending `protocol` as `Sec-WebSocket-Protocol`.
///
/// endpoint-libs servers authenticate in that header: `0<endpoint>, 1<param>, 2<param>...`,
/// with the connect endpoint's name lowercased and its parameters in schema
/// order. See [`protocol_header`].
pub async fn connect(url: &str, protocol: &str, handle: &Handle) -> Result<WsClient> {
    let (client, _) = WsClientBuilder::new()
        .protocol_header(protocol)
        .build(url, handle)
        .await
        .wrap_err_with(|| format!("could not connect to {url}"))?;
    Ok(client)
}

/// The handshake header for a connect endpoint and its parameter values, in
/// schema order. Each value is numbered by its position, so a missing
/// optional one is left out without moving the ones after it.
pub fn protocol_header(endpoint: &str, params: &[Option<String>]) -> String {
    std::iter::once(format!("0{}", endpoint.to_ascii_lowercase()))
        .chain(
            params
                .iter()
                .enumerate()
                .filter_map(|(i, value)| value.as_ref().map(|value| format!("{}{value}", i + 1))),
        )
        .collect::<Vec<_>>()
        .join(", ")
}

/// The next frame, or an error after [`RECV_TIMEOUT`].
pub async fn recv(client: &mut WsClient) -> Result<serde_json::Value> {
    match recv_within(client, RECV_TIMEOUT).await? {
        Some(frame) => Ok(frame),
        None => bail!("no frame within {}s", RECV_TIMEOUT.as_secs()),
    }
}

/// The next frame if one arrives within `wait`. Frames are buffered by the
/// connection, so giving up leaves nothing half read.
pub async fn recv_within(
    client: &mut WsClient,
    wait: Duration,
) -> Result<Option<serde_json::Value>> {
    let frame = client.recv_raw().fuse();
    let timeout = nagoya::sleep(wait).fuse();
    futures::pin_mut!(frame, timeout);
    match select(frame, timeout).await {
        Either::Left((frame, _)) => frame.map(Some),
        Either::Right(_) => Ok(None),
    }
}

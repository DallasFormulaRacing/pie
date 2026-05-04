use tokio_tungstenite::tungstenite::Message as WsFrame;

use super::BackendEvent;

#[derive(Debug, thiserror::Error)]
pub enum WsError {
    #[error("websocket JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn encode_outgoing(message: &BackendEvent) -> Result<WsFrame, WsError> {
    Ok(WsFrame::Text(serde_json::to_string(message)?.into()))
}

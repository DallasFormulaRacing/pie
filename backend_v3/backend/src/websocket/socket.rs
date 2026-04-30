use tokio_tungstenite::tungstenite::Message as WsFrame;

use super::{WsIncoming, WsOutgoing};

#[derive(Debug, thiserror::Error)]
pub enum WsError {
    #[error("websocket JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn decode_incoming(frame: WsFrame) -> Result<Option<WsIncoming>, WsError> {
    let WsFrame::Text(text) = frame else {
        return Ok(None);
    };

    Ok(Some(serde_json::from_str(&text)?))
}

pub fn encode_outgoing(message: &WsOutgoing) -> Result<WsFrame, WsError> {
    Ok(WsFrame::Text(serde_json::to_string(message)?.into()))
}

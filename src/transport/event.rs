use axum::response::sse::Event;
use rmcp::model;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MCPEvent {
    Endpoint(String),
    Message(model::JsonRpcMessage),
}

impl From<model::JsonRpcMessage> for MCPEvent {
    fn from(message: model::JsonRpcMessage) -> Self {
        MCPEvent::Message(message)
    }
}

impl From<(model::NumberOrString, std::io::Error)> for MCPEvent {
    fn from((id, error): (model::NumberOrString, std::io::Error)) -> Self {
        MCPEvent::Message(model::JsonRpcMessage::Error(model::JsonRpcError {
            id: id.clone(),
            jsonrpc: model::JsonRpcVersion2_0,
            error: model::ErrorData {
                code: model::ErrorCode::INTERNAL_ERROR,
                message: error.to_string().into(),
                data: None,
            },
        }))
    }
}

impl From<MCPEvent> for Event {
    fn from(event: MCPEvent) -> Self {
        match event {
            MCPEvent::Endpoint(endpoint) => Event::default().event("endpoint").data(endpoint),
            MCPEvent::Message(message) => Event::default()
                .event("message")
                .data(serde_json::to_string(&message).unwrap()),
        }
    }
}

impl From<Error> for model::JsonRpcMessage {
    fn from(error: Error) -> Self {
        model::JsonRpcMessage::Error(model::JsonRpcError {
            id: model::NumberOrString::Number(u32::MAX),
            jsonrpc: model::JsonRpcVersion2_0,
            error: model::ErrorData {
                code: model::ErrorCode(-32603),
                data: None,
                message: error.to_string().into(),
            },
        })
    }
}

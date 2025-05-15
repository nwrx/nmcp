use axum::response::sse::Event;
use rmcp::model;
use serde_json as JSON;

#[derive(Clone, Debug)]
pub enum MCPEvent {
    Endpoint(String),
    Message(model::JsonRpcMessage),
    Error(String),
}

impl From<model::JsonRpcMessage> for MCPEvent {
    fn from(message: model::JsonRpcMessage) -> Self {
        MCPEvent::Message(message)
    }
}

impl From<MCPEvent> for Event {
    fn from(event: MCPEvent) -> Self {
        match event {
            MCPEvent::Endpoint(endpoint) => Event::default().event("endpoint").data(endpoint),
            MCPEvent::Error(error) => Event::default().event("error").data(error),
            MCPEvent::Message(message) => Event::default()
                .event("message")
                .data(JSON::to_string(&message).unwrap()),
        }
    }
}

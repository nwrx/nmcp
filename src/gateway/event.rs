use crate::ErrorInner;
use rmcp::model;

impl From<ErrorInner> for model::JsonRpcMessage {
    fn from(error: ErrorInner) -> Self {
        Self::Error(model::JsonRpcError {
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

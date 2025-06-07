use super::{Error, ErrorInner};
use std::any::Any;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error
where
    T: Any + Send + Sync + 'static,
{
    fn from(source: tokio::sync::mpsc::error::SendError<T>) -> Self {
        let message = format!("Failed to send on MPSC channel: {source}");
        let payload: Box<dyn Any + Send + Sync> = Box::new(source.0);
        let source = tokio::sync::mpsc::error::SendError(payload);
        let inner = ErrorInner::MPSCSendError(source);
        Self::new(inner)
            .with_name("E_TOKIO_MPSC_SEND_ERROR")
            .with_message(message)
    }
}

impl<T> From<tokio::sync::broadcast::error::SendError<T>> for Error
where
    T: Any + Send + Sync + 'static,
{
    fn from(source: tokio::sync::broadcast::error::SendError<T>) -> Self {
        let message = format!("Failed to send on Broadcast channel: {source}");
        let payload: Box<dyn Any + Send + Sync> = Box::new(source.0);
        let source = tokio::sync::broadcast::error::SendError(payload);
        let inner = ErrorInner::BroadcastSendError(source);
        Self::new(inner)
            .with_name("E_TOKIO_BROADCAST_SEND_ERROR")
            .with_message(message)
    }
}

// RecvError
impl From<tokio::sync::broadcast::error::RecvError> for Error {
    fn from(source: tokio::sync::broadcast::error::RecvError) -> Self {
        let message = format!("Failed to receive on MPSC channel: {source}");
        match source {
            tokio::sync::broadcast::error::RecvError::Lagged(_) => {
                let inner = ErrorInner::BroadcastRecvError(source);
                Self::new(inner)
                    .with_name("E_TOKIO_BROADCAST_RECV_ERROR")
                    .with_message(message)
            }
            tokio::sync::broadcast::error::RecvError::Closed => {
                let inner = ErrorInner::BroadcastRecvError(source);
                Self::new(inner)
                    .with_name("E_TOKIO_BROADCAST_RECV_CLOSED")
                    .with_message(message)
            }
        }
    }
}

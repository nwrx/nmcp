use super::{Error, ErrorInner};
use axum::http::StatusCode;

impl From<kube::Error> for Error {
    fn from(source: kube::Error) -> Self {
        match &source {
            kube::Error::Api(error) => {
                let code = error.code;
                let status_text = StatusCode::from_u16(code)
                    .unwrap_or_default()
                    .canonical_reason()
                    .unwrap_or_default()
                    .replace(' ', "_")
                    .to_uppercase();
                let name = format!("E_KUBE_API_{status_text}");
                let source = ErrorInner::KubeError(source);
                Self::new(source).with_name(name).with_status(code)
            }

            // Failure to build a request to the Kubernetes API.
            kube::Error::BuildRequest(..) => {
                let source = ErrorInner::KubeError(source);
                Self::new(source).with_name("E_KUBE_BUILD_REQUEST")
            }

            // This error occurs when trying to attach to a pod.
            kube::Error::UpgradeConnection(error) => match error {
                kube::client::UpgradeConnectionError::ProtocolSwitch(code) => {
                    let code = *code;
                    let source = ErrorInner::KubeError(source);
                    Self::new(source)
                        .with_name("E_KUBE_UPGRADE_CONNECTION_PROTOCOL_SWITCH")
                        .with_status(code)
                }
                kube::client::UpgradeConnectionError::MissingUpgradeWebSocketHeader => {
                    let source = ErrorInner::KubeError(source);
                    Self::new(source).with_name("E_KUBE_UPGRADE_CONNECTION_MISSING_UPGRADE_HEADER")
                }
                kube::client::UpgradeConnectionError::MissingConnectionUpgradeHeader => {
                    let source = ErrorInner::KubeError(source);
                    Self::new(source)
                        .with_name("E_KUBE_UPGRADE_CONNECTION_MISSING_CONNECTION_HEADER")
                }
                kube::client::UpgradeConnectionError::SecWebSocketAcceptKeyMismatch => {
                    let source = ErrorInner::KubeError(source);
                    Self::new(source)
                        .with_name("E_KUBE_UPGRADE_CONNECTION_SEC_WEBSOCKET_ACCEPT_KEY_MISMATCH")
                }
                kube::client::UpgradeConnectionError::SecWebSocketProtocolMismatch => {
                    let source = ErrorInner::KubeError(source);
                    Self::new(source)
                        .with_name("E_KUBE_UPGRADE_CONNECTION_SEC_WEBSOCKET_PROTOCOL_MISMATCH")
                }
                kube::client::UpgradeConnectionError::GetPendingUpgrade(..) => {
                    let source = ErrorInner::KubeError(source);
                    Self::new(source).with_name("E_KUBE_UPGRADE_CONNECTION_GET_PENDING_UPGRADE")
                }
            },
            _ => {
                let message = format!("{source:?}");
                let source = ErrorInner::KubeError(source);
                Self::new(source)
                    .with_name("E_KUBE_API")
                    .with_message(message)
            }
        }
    }
}

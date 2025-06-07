use super::{Error, ErrorInner};

impl From<kube::runtime::watcher::Error> for Error {
    fn from(source: kube::runtime::watcher::Error) -> Self {
        match source {
            kube::runtime::watcher::Error::InitialListFailed(error) => {
                let error = ErrorInner::KubeError(error);
                Self::new(error)
                    .with_name("E_KUBE_WATCHER_INITIAL_LIST_FAILED")
                    .with_message("Failed to list initial resources".to_string())
            }
            kube::runtime::watcher::Error::WatchStartFailed(error) => {
                let error = ErrorInner::KubeError(error);
                Self::new(error)
                    .with_name("E_KUBE_WATCHER_START_FAILED")
                    .with_message("Failed to start watcher".to_string())
            }
            kube::runtime::watcher::Error::WatchError(response) => {
                let message = response.message.clone();
                Self::generic(message)
                    .with_name("E_KUBE_WATCHER_ERROR")
                    .with_status(response.code)
            }
            kube::runtime::watcher::Error::WatchFailed(error) => {
                let error = ErrorInner::KubeError(error);
                Self::new(error)
                    .with_name("E_KUBE_WATCHER_FAILED")
                    .with_message("Watcher failed".to_string())
            }
            kube::runtime::watcher::Error::NoResourceVersion => {
                Self::generic("No resource version provided for watcher")
                    .with_name("E_KUBE_WATCHER_NO_RESOURCE_VERSION")
            }
        }
    }
}

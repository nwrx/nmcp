use super::{Error, ErrorInner};

impl<ReconcilerErr>
    From<kube::runtime::controller::Error<ReconcilerErr, kube::runtime::watcher::Error>> for Error
where
    ReconcilerErr: std::error::Error + Send + Sync + 'static,
{
    fn from(
        source: kube::runtime::controller::Error<ReconcilerErr, kube::runtime::watcher::Error>,
    ) -> Self {
        match source {
            kube::runtime::controller::Error::ObjectNotFound(object) => {
                let message = format!("Object not found: {object:?}");
                let source = ErrorInner::Generic(message.clone());
                Self::new(source).with_name("E_KUBE_CONTROLLER_OBJECT_NOT_FOUND")
            }
            kube::runtime::controller::Error::ReconcilerFailed(error, object) => {
                let message = error.to_string();
                let message = format!("{object:?}: {message}");
                Self::generic(message).with_name("E_KUBE_CONTROLLER_RECONCILER_ERROR")
            }
            kube::runtime::controller::Error::QueueError(error) => {
                Self::from(error).with_name("E_KUBE_CONTROLLER_QUEUE_ERROR")
            }
            kube::runtime::controller::Error::RunnerError(error) => {
                let message = format!("{error}");
                Self::generic(message).with_name("E_KUBE_CONTROLLER_RECONCILER_PANIC")
            }
        }
    }
}

use super::{Error, ErrorInner};

impl<T> From<kube::runtime::finalizer::Error<T>> for Error
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn from(source: kube::runtime::finalizer::Error<T>) -> Self {
        match source {
            kube::runtime::finalizer::Error::AddFinalizer(error) => {
                let source = ErrorInner::KubeClientError(error);
                Self::new(source).with_name("E_KUBE_FINALIZER_ADD")
            }
            kube::runtime::finalizer::Error::RemoveFinalizer(error) => {
                let source = ErrorInner::KubeClientError(error);
                Self::new(source).with_name("E_KUBE_FINALIZER_REMOVE")
            }
            kube::runtime::finalizer::Error::ApplyFailed(error) => {
                let message = format!("Failed to apply finalizer: {error}");
                Self::generic(message).with_name("E_KUBE_FINALIZER_APPLY_FAILED")
            }
            kube::runtime::finalizer::Error::CleanupFailed(error) => {
                let message = format!("Failed to clean up finalizer: {error}");
                Self::generic(message).with_name("E_KUBE_FINALIZER_CLEANUP_FAILED")
            }
            kube::runtime::finalizer::Error::UnnamedObject => {
                Self::generic("Unnamed object in finalizer error")
                    .with_name("E_KUBE_FINALIZER_UNNAMED_OBJECT")
            }
            kube::runtime::finalizer::Error::InvalidFinalizer => {
                Self::generic("Invalid finalizer in finalizer error")
                    .with_name("E_KUBE_FINALIZER_INVALID_FINALIZER")
            }
        }
    }
}

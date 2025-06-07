use std::future::Future;
use std::sync::Arc;
use tokio::task::JoinError;
use tokio::task::JoinHandle;

#[derive(Debug)]
enum TaskState<V, E> {
    Pending(JoinHandle<Result<V, E>>),
    Aborted(JoinError),
    Rejected(Arc<E>),
    Resolved(Arc<V>),
}

impl<V, E> Drop for TaskState<V, E> {
    fn drop(&mut self) {
        if let Self::Pending(handle) = self {
            handle.abort();
        }
    }
}

impl<V, E> From<Result<V, E>> for TaskState<V, E> {
    fn from(result: Result<V, E>) -> Self {
        match result {
            Ok(value) => Self::Resolved(value.into()),
            Err(error) => Self::Rejected(error.into()),
        }
    }
}

impl<V, E> From<JoinError> for TaskState<V, E> {
    fn from(error: JoinError) -> Self {
        Self::Aborted(error)
    }
}

impl<V, E> From<JoinHandle<Result<V, E>>> for TaskState<V, E> {
    fn from(handle: JoinHandle<Result<V, E>>) -> Self {
        Self::Pending(handle)
    }
}

impl<V, E, F> From<F> for Task<V, E>
where
    F: Future<Output = Result<V, E>> + Send + 'static,
    V: Send + 'static,
    E: Send + 'static,
{
    fn from(future: F) -> Self {
        let handle = tokio::spawn(future);
        Self(TaskState::Pending(handle))
    }
}

/// Thread-safe collection of tasks that can be used to cache results of asynchronous computations.
/// This struct wraps a `JoinHandle` that can resolve to a value or an error. Each task can be in
/// one of the following states: `Pending`, `Aborted`, `Error`, or `Value`.
#[derive(Debug)]
pub struct Task<V, E>(TaskState<V, E>);

impl<V, E> Task<V, E> {
    pub fn value(value: V) -> Self {
        Self(TaskState::Resolved(Arc::new(value)))
    }

    /// Set the inner state of the task with a new value or error.
    ///
    /// This method allows you to update the task's state directly, which can be useful
    /// for setting the result of a computation or an error that occurred during the task's execution.
    /// The `inner` parameter can be any type that implements `Into<TaskInner<V, E>>`, allowing
    /// for flexibility in how the task's state is set.
    ///
    fn set(&mut self, inner: impl Into<TaskState<V, E>>) -> &mut Self {
        self.0 = inner.into();
        self
    }

    /// Inspect the current state of the task and wait for it to complete if it's pending.
    ///
    /// This method will block until the task is resolved, either with a value or an error,
    /// and then update the task's state accordingly. If a `JoinError` occurs, it will be set
    /// as the task's state.
    ///
    pub async fn resolve(&mut self) -> &Self {
        if let TaskState::Pending(handle) = &mut self.0 {
            match handle.await {
                Ok(result) => {
                    let _ = self.set(result);
                }
                Err(error) => {
                    let _ = self.set(error);
                }
            };
        };
        self
    }

    /// Peek at the current result of the task without resolving it.
    ///
    /// This method returns an `Option` containing either the value or the error if the
    /// task has already been resolved. Note that if the task is still pending or was aborted,
    /// it will return `None`.
    ///
    pub async fn peek(&self) -> Option<Result<Arc<V>, Arc<E>>> {
        match &self.0 {
            TaskState::Resolved(value) => Some(Ok(value.clone())),
            TaskState::Rejected(error) => Some(Err(error.clone())),
            _ => None,
        }
    }

    /// Get the `JoinError` if the task was aborted.
    pub fn aborted(&self) -> Option<&JoinError> {
        match &self.0 {
            TaskState::Aborted(error) => Some(error),
            _ => None,
        }
    }
}

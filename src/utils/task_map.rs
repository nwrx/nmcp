use super::Task;
use std::fmt::Debug;
use std::future::Future;
use std::{collections::HashMap, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, RwLock};

/// Type alias for a task entry that can be stored in the `TaskMap`.
pub type TaskEntry<V, E> = Arc<RwLock<Task<V, E>>>;

/// A concurrent map that can store values or tasks that will resolve to values.
/// This is useful for caching async computations and avoiding duplicate work.
#[derive(Clone)]
pub struct TaskMap<K, V, E = ()> {
    inner: Arc<Mutex<HashMap<K, TaskEntry<V, E>>>>,
    length: usize,
}

impl Default for TaskMap<String, String, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for TaskMap<String, String, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.inner.blocking_lock();
        f.debug_struct("TaskMap")
            .field("len", &guard.len())
            .field("is_empty", &guard.is_empty())
            .finish()
    }
}

impl<K, V, E> TaskMap<K, V, E>
where
    K: Clone + Eq + Hash + 'static,
{
    /// Create a new empty `HashMapAsync`
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            length: 0,
        }
    }

    /// The length of the map, i.e., the number of keys.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if a key exists in either values or tasks
    pub async fn contains_key(&self, key: &K) -> bool {
        let guard = self.inner.lock().await;
        let result = guard.get(key);
        result.is_some()
            && match result {
                Some(task) => task.write().await.resolved().await.peek().await.is_some(),
                None => false,
            }
    }

    /// Get the result of a task if it exists.
    pub async fn get_ref(&mut self, key: &K) -> Option<Result<Arc<V>, Arc<E>>> {
        let guard = self.inner.lock().await;
        match guard.get(key) {
            None => None,
            Some(task) => task.write().await.resolved().await.peek().await,
        }
    }

    /// Set a value for a key, replacing any existing value.
    pub async fn set(&mut self, key: K, value: V) -> Result<Arc<V>, Arc<E>> {
        let mut guard = self.inner.lock().await;
        let task = Task::value(value);
        let task = Arc::new(RwLock::new(task));
        let _ = guard.insert(key.clone(), task.clone());
        let result = task.write().await.peek().await.unwrap();
        self.length = guard.len();
        result
    }

    /// Set a task for a key, replacing any existing task.
    pub async fn insert(
        &mut self,
        key: K,
        task: impl Into<Task<V, E>>,
    ) -> Option<Result<Arc<V>, Arc<E>>> {
        let mut guard = self.inner.lock().await;
        let task = Arc::new(RwLock::new(task.into()));
        let _ = guard.insert(key.clone(), task.clone());
        let result = task.write().await.resolved().await.peek().await;
        self.length = guard.len();
        result
    }

    /// Get a value or set it if it doesn't exist.
    pub async fn get_ref_or_set(&mut self, key: &K, value: V) -> Result<Arc<V>, Arc<E>> {
        let mut guard = self.inner.lock().await;

        // --- Check if the key exists and the value is Some.
        if let Some(task) = guard.get(key) {
            if let Some(result) = task.write().await.resolved().await.peek().await {
                return result;
            }
        }

        // --- If the key does not exist or the value is None, create a new task.
        let task = Task::value(value);
        let task = Arc::new(RwLock::new(task));
        let _ = guard.insert(key.clone(), task.clone());

        // --- Return the value of the task directly.
        let result = task.write().await.peek().await.unwrap();
        result
    }

    /// Get a value or compute it if it doesn't exist.
    pub async fn get_ref_or_insert<F, Fut>(
        &mut self,
        key: &K,
        task_fn: F,
    ) -> Option<Result<Arc<V>, Arc<E>>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
        V: Send + 'static,
        E: Send + 'static,
    {
        let mut tasks = self.inner.lock().await;

        // --- Check if the key exists and the value is Some.
        if let Some(task) = tasks.get(key) {
            if let Some(result) = task.write().await.resolved().await.peek().await {
                return Some(result);
            }
        }

        // --- If the key does not exist or the value is None, create a new task.
        let task = Task::from(task_fn());
        let task = Arc::new(RwLock::new(task));
        let _ = tasks.insert(key.clone(), task.clone());

        // --- Wait for the task to resolve and return its value.
        let result = task.write().await.resolved().await.peek().await;
        result
    }

    /// Remove a key from the map and return its value or error.
    pub async fn remove(&mut self, key: &K) -> Option<Result<Arc<V>, Arc<E>>> {
        let mut guard = self.inner.lock().await;
        let result = match guard.remove(key) {
            None => None,
            Some(task) => task.write().await.resolved().await.peek().await,
        };
        self.length = guard.len();
        result
    }

    /// Discard a key from the map without returning its value or error.
    pub async fn discard(&mut self, key: &K) {
        let mut guard = self.inner.lock().await;
        let value = guard.remove(key);
        drop(value);
        self.length = guard.len();
    }

    /// Iterate over the entries in the map, yielding key-value pairs.
    pub async fn iter_tasks(&self) -> impl Iterator<Item = (K, TaskEntry<V, E>)> {
        let guard = self.inner.lock();
        guard
            .await
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<K, V, E> TaskMap<K, V, E>
where
    K: Clone + Eq + Hash + 'static,
    V: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    /// Get the result of a task if it exists and get owned clone of the underlying value.
    pub async fn get(&mut self, key: &K) -> Option<Result<V, E>> {
        let guard = self.inner.lock().await;
        match guard.get(key) {
            None => None,
            Some(task) => {
                let result = task.write().await.resolved().await.peek().await;
                match result {
                    Some(Ok(value)) => Some(Ok((*value).clone())),
                    Some(Err(error)) => Some(Err((*error).clone())),
                    None => None,
                }
            }
        }
    }

    /// Get a value or set it if it doesn't exist, returning an owned clone of the value.
    pub async fn get_or_set(&mut self, key: &K, value: V) -> Result<V, E> {
        self.get_ref_or_set(key, value)
            .await
            .map(|v| (*v).clone())
            .map_err(|e| (*e).clone())
    }

    /// Get a value or compute it if it doesn't exist, returning an owned clone of the value.
    pub async fn get_or_insert<F, Fut>(&mut self, key: &K, task_fn: F) -> Option<Result<V, E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
        V: Send + 'static,
        E: Send + 'static,
    {
        self.get_ref_or_insert(key, task_fn).await.map(|v| match v {
            Ok(value) => Ok((*value).clone()),
            Err(error) => Err((*error).clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_insert_and_get_value_once() {
        let mut tasks = TaskMap::<&str, i32, ()>::new();
        let _ = tasks.set("key1", 42).await;
        let value = tasks.get_ref(&"key1").await.unwrap().unwrap();
        assert_eq!(*value, 42);
    }

    #[tokio::test]
    async fn test_insert_and_get_value_twice() {
        let mut tasks = TaskMap::<&str, i32, ()>::new();
        let _ = tasks.set("key1", 42).await;
        let value1 = tasks.get_ref(&"key1").await;
        let value2 = tasks.get_ref(&"key1").await;
        assert_eq!(*value1.unwrap().unwrap(), 42);
        assert_eq!(*value2.unwrap().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_insert_and_get_task() {
        let mut tasks = TaskMap::<&str, i32, ()>::new();
        let _ = tasks
            .insert("key1", async {
                sleep(Duration::from_millis(50)).await;
                Ok::<i32, ()>(100)
            })
            .await;
        let value = tasks.get_ref(&"key1").await.unwrap().unwrap();
        assert_eq!(*value, 100);
    }

    #[tokio::test]
    async fn test_insert_and_get_task_twice() {
        let mut task = TaskMap::<&str, i32, ()>::new();
        let _ = task
            .insert("key1", async {
                sleep(Duration::from_millis(50)).await;
                Ok(100)
            })
            .await;
        let value1 = task.get_ref(&"key1").await;
        let value2 = task.get_ref(&"key1").await;
        assert_eq!(*value1.unwrap().unwrap(), 100);
        assert_eq!(*value2.unwrap().unwrap(), 100);
    }

    // #[tokio::test]
    // async fn test_insert_and_get_task_with_error() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let task = tokio::spawn(async {
    //         sleep(Duration::from_millis(50)).await;
    //         Err(())
    //     });
    //     map.insert_task("key1", task).await;
    //     let result = map.get_task(&"key1").await.unwrap();
    //     assert!(result.is_err());
    // }

    // #[tokio::test]
    // async fn test_insert_and_get_task_with_error_twice() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let task = tokio::spawn(async {
    //         sleep(Duration::from_millis(50)).await;
    //         Err(())
    //     });
    //     map.insert_task("key1", task).await;
    //     let result1 = map.get_task(&"key1");
    //     let result2 = map.get_task(&"key1");
    //     assert!(result1.await.unwrap().is_err());
    //     assert!(result2.await.is_none());
    // }

    // #[tokio::test]
    // async fn test_remove_key() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     map.insert_value("key1", 42).await;
    //     assert!(map.remove(&"key1").await);
    //     assert!(!map.contains_key(&"key1").await);
    // }

    #[tokio::test]
    async fn test_get_or_spawn_avoids_duplicate_tasks() {
        let tasks = TaskMap::<&str, i32, ()>::new();
        let mut tasks1 = tasks.clone();
        let mut tasks2 = tasks.clone();
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter1 = counter.clone();
        let counter2 = counter.clone();

        // Create two clones of the map to simulate concurrent access
        // Spawn two tasks that both try to compute the same value
        let task1 = tokio::spawn(async move {
            tasks1
                .get_ref_or_insert(&"key1", || async move {
                    let _ = counter1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    sleep(Duration::from_millis(50)).await;
                    Ok(100)
                })
                .await
        });

        // Add a small delay to make sure the first task starts first
        sleep(Duration::from_millis(1)).await;
        let task2 = tokio::spawn(async move {
            tasks2
                .get_ref_or_insert(&"key1", || async move {
                    let _ = counter2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    sleep(Duration::from_millis(50)).await;
                    Ok(200) // Different value to show we're using the first task's result
                })
                .await
        });

        // Both tasks should complete with the same value from the first computation
        let result1 = task1.await.unwrap().unwrap().unwrap();
        let result2 = task2.await.unwrap().unwrap().unwrap();

        assert_eq!(*result1, 100);
        assert_eq!(*result2, 100);
        // The computation function should only have been called once
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // #[tokio::test]
    // async fn test_get_or_compute() {
    //     let map = TaskMap::<&str, i32, ()>::new();

    //     let result = map
    //         .get_or_compute("key1", || async {
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(42)
    //         })
    //         .await;

    //     assert_eq!(result, Ok(42));

    //     // Check that the value is now cached
    //     let cached_result = map.get_task(&"key1").await.unwrap().unwrap();
    //     assert_eq!(cached_result, 42);
    // }

    // #[tokio::test]
    // async fn test_get_or_compute_avoids_duplicate_tasks() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    //     // Create two clones of the map to simulate concurrent access
    //     let map1 = map.clone();
    //     let map2 = map.clone();
    //     let counter1 = counter.clone();
    //     let counter2 = counter.clone();

    //     // Spawn two tasks that both try to compute the same value
    //     let task1 = tokio::spawn(async move {
    //         map1.get_or_spawn("key1", || async move {
    //             counter1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(100)
    //         })
    //         .await
    //     });

    //     // Add a small delay to make sure the first task starts first
    //     sleep(Duration::from_millis(10)).await;

    //     let task2 = tokio::spawn(async move {
    //         map2.get_or_spawn("key1", || async move {
    //             counter2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(200) // Different value to show we're using the first task's result
    //         })
    //         .await
    //     });

    //     // Both tasks should complete with the same value from the first computation
    //     let result1 = task1.await.unwrap();
    //     let result2 = task2.await.unwrap();

    //     assert_eq!(result1, Ok(100));
    //     assert_eq!(result2, Ok(100));
    //     // The computation function should only have been called once
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    // }

    // #[tokio::test]
    // async fn test_get_or_insert() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    //     let counter_clone = counter.clone();

    //     // Test with pre-created task
    //     let task = tokio::spawn(async move {
    //         counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //         sleep(Duration::from_millis(50)).await;
    //         Ok(100)
    //     });

    //     let result1 = map.get_or_spawn("key1", task).await;
    //     assert_eq!(result1, Ok(100));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    //     // Test with function returning task
    //     let counter_clone = counter.clone();
    //     let result2 = map
    //         .get_or_spawn("key2", || {
    //             let counter = counter_clone.clone();
    //             tokio::spawn(async move {
    //                 counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 sleep(Duration::from_millis(50)).await;
    //                 Ok(200)
    //             })
    //         })
    //         .await;

    //     assert_eq!(result2, Ok(200));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);

    //     // Test with async closure
    //     let counter_clone = counter.clone();
    //     let result3 = map
    //         .get_or_spawn("key3", || async move {
    //             counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(300)
    //         })
    //         .await;

    //     assert_eq!(result3, Ok(300));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);

    //     // Test that values are cached and functions not called again
    //     let counter_clone = counter.clone();
    //     let result1_again = map
    //         .get_or_spawn("key1", || async move {
    //             counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(999) // Should not be used
    //         })
    //         .await;

    //     assert_eq!(result1_again, Ok(100)); // Original value
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3); // Counter unchanged
    // }

    // #[tokio::test]
    // async fn test_compute_if_absent() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    //     let counter_clone = counter.clone();

    //     // Test computing a value that doesn't exist
    //     let result = map
    //         .get_or_compute("key1", || async move {
    //             counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(42)
    //         })
    //         .await;

    //     assert_eq!(result, Ok(42));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    //     // Test that the value is cached and the function isn't called again
    //     let counter_clone = counter.clone();
    //     let result2 = map
    //         .get_or_compute("key1", || async move {
    //             counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //             sleep(Duration::from_millis(50)).await;
    //             Ok(99) // Different value to show we're using the cached result
    //         })
    //         .await;

    //     assert_eq!(result2, Ok(42)); // Should return the original value
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1); // Counter unchanged
    // }

    // #[tokio::test]
    // async fn test_get_or_set() {
    //     let map = TaskMap::<&str, i32, ()>::new();

    //     // Test setting a value that doesn't exist
    //     let result = map.get_or_set("key1", 42).await;
    //     assert_eq!(result, 42);

    //     // Test that the existing value is returned
    //     let result2 = map.get_or_set("key1", 99).await;
    //     assert_eq!(result2, 42); // Should return the original value

    //     // Verify through get
    //     let cached_result = map.get_task(&"key1").await.unwrap().unwrap();
    //     assert_eq!(cached_result, 42);
    // }

    // #[tokio::test]
    // async fn test_get_or_compute_with_direct_value() {
    //     let map = TaskMap::<&str, i32, ()>::new();

    //     // Test with direct value
    //     let result = map.get_or_compute("key1", 42).await;
    //     assert_eq!(result, Ok(42));

    //     // Verify it's stored
    //     let cached = map.get_task(&"key1").await.unwrap().unwrap();
    //     assert_eq!(cached, 42);
    // }

    // #[tokio::test]
    // async fn test_get_or_compute_with_result() {
    //     let map = TaskMap::<&str, i32, ()>::new();

    //     // Test with Result
    //     let result = map.get_or_compute("key1", Ok::<_, ()>(42)).await;
    //     assert_eq!(result, Ok(42));

    //     // Test with Err result
    //     let result = map.get_or_compute("key2", Err::<i32, _>(())).await;
    //     assert!(result.is_err());
    // }

    // #[tokio::test]
    // async fn test_get_or_compute_with_sync_fn() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    //     let counter_clone = counter.clone();

    //     // Test with sync function using the SyncFn wrapper
    //     let result = map
    //         .get_or_compute(
    //             "key1",
    //             SyncFn(|| {
    //                 counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 Ok::<_, ()>(42)
    //             }),
    //         )
    //         .await;

    //     assert_eq!(result, Ok(42));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    //     // Check that it's cached
    //     let counter_clone = counter.clone();
    //     let result2 = map
    //         .get_or_compute(
    //             "key1",
    //             SyncFn(|| {
    //                 counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                 Ok::<_, ()>(99) // Different value
    //             }),
    //         )
    //         .await;

    //     assert_eq!(result2, Ok(42)); // Original value
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1); // Not called
    // }

    // #[tokio::test]
    // async fn test_get_or_compute_with_task_fn() {
    //     let map = TaskMap::<&str, i32, ()>::new();
    //     let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    //     let counter_clone = counter.clone();

    //     // Test with TaskFn wrapper
    //     let result = map
    //         .get_or_compute(
    //             "key1",
    //             TaskFn(|| {
    //                 let counter = counter_clone.clone();
    //                 tokio::spawn(async move {
    //                     counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                     sleep(Duration::from_millis(50)).await;
    //                     Ok(42)
    //                 })
    //             }),
    //         )
    //         .await;

    //     assert_eq!(result, Ok(42));
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    //     // Check that it's cached
    //     let counter_clone = counter.clone();
    //     let result2 = map
    //         .get_or_compute(
    //             "key1",
    //             TaskFn(|| {
    //                 let counter = counter_clone.clone();
    //                 tokio::spawn(async move {
    //                     counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    //                     sleep(Duration::from_millis(50)).await;
    //                     Ok(99) // Different value
    //                 })
    //             }),
    //         )
    //         .await;

    //     assert_eq!(result2, Ok(42)); // Original value
    //     assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1); // Not called
    // }
}

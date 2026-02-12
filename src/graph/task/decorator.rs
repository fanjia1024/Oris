use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::graph::task::{FunctionTask, TaskError};

/// Helper function to create a task from an async function
///
/// This is a convenience function for creating tasks. For more control,
/// use `FunctionTask::new` directly.
pub fn task<F, Fut>(
    task_id: impl Into<String>,
    func: F,
) -> FunctionTask<
    impl Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, TaskError>> + Send>>
        + Send
        + Sync
        + 'static,
>
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Value, TaskError>> + Send + 'static,
{
    let wrapped =
        move |v: Value| -> Pin<Box<dyn Future<Output = Result<Value, TaskError>> + Send>> {
            Box::pin(func(v))
        };
    FunctionTask::new(task_id, wrapped)
}

/// Macro to create a task from a function
///
/// # Example
///
/// ```ignore
/// use oris::graph::task::task;
/// use serde_json::Value;
/// let _task = task("my_task", |_input: Value| async move { Ok(serde_json::json!("result")) });
/// ```
#[macro_export]
macro_rules! create_task {
    ($task_id:expr, $func:expr) => {
        $crate::graph::task::decorator::task($task_id, $func)
    };
}

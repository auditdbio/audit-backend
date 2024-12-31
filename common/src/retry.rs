use std::{
    future::Future,
    pin::Pin,
    time::Duration,
    sync::Arc,
};
use tokio::{time::sleep, sync::Mutex};

use crate::error;

async fn retry_async<F, T>(mut operation: F) -> error::Result<T>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = error::Result<T>> + Send>>,
        T: Send + 'static,
{
    const MAX_RETRIES: usize = 3;
    for attempt in 0..MAX_RETRIES {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.code == 409 && attempt < MAX_RETRIES - 1 => {
                eprintln!("Attempt {} failed: {:?}. Retrying...", attempt + 1, e);
                sleep(Duration::from_millis(100)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!();
}

pub async fn retry_operation<F, Fut, T>(operation: F) -> error::Result<T>
    where
        F: Fn() -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = error::Result<T>> + Send + 'static,
        T: Send + 'static,
{
    let operation = Arc::new(Mutex::new(operation));
    retry_async(|| {
        let op = operation.clone();
        Box::pin(async move {
            let op = op.lock().await;
            op().await
        }) as Pin<Box<dyn Future<Output = error::Result<T>> + Send>>
    }).await
}
//! Target-specific timers shared by federation and browser discovery.
#[cfg(not(target_arch = "wasm32"))]
pub use tokio::time::{sleep, timeout};

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: std::time::Duration) {
    // Browser timers use signed 32-bit milliseconds. Split larger durations.
    let mut remaining = duration.as_millis();
    loop {
        let millis = remaining.min(i32::MAX as u128) as u32;
        gloo_timers::future::TimeoutFuture::new(millis).await;
        remaining -= u128::from(millis);
        if remaining == 0 {
            break;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn timeout<F: std::future::Future>(
    duration: std::time::Duration,
    future: F,
) -> Result<F::Output, &'static str> {
    use futures::future::{select, Either};
    match select(Box::pin(future), Box::pin(sleep(duration))).await {
        Either::Left((output, _)) => Ok(output),
        Either::Right(_) => Err("request timed out"),
    }
}

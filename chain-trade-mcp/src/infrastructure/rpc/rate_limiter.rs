use governor::{DefaultDirectRateLimiter, Quota};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

pub struct RpcRateLimiter {
    inner: Arc<DefaultDirectRateLimiter>,
}

impl Clone for RpcRateLimiter {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl RpcRateLimiter {
    pub fn new(max_tokens: u64, refill_interval_ms: u64) -> Self {
        let quota = Quota::with_period(Duration::from_millis(refill_interval_ms))
            .expect("Valid quota")
            .allow_burst(NonZeroU32::new(max_tokens as u32).expect("Non-zero max tokens"));

        let clock = governor::clock::DefaultClock::default();
        let limiter = governor::RateLimiter::direct_with_clock(quota, &clock);

        Self {
            inner: Arc::new(limiter),
        }
    }

    pub fn check(&self) -> bool {
        self.inner.check().is_ok()
    }

    #[allow(dead_code)]
    pub async fn acquire(&self) -> bool {
        self.check()
    }
}

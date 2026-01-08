use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

fn now_ms() -> u64 {
    Instant::now().elapsed().as_millis() as u64
}

#[derive(Clone)]
pub struct RateLimiter {
    tokens: Arc<AtomicU64>,
    last_refill: Arc<AtomicU64>,
    max_tokens: u64,
    refill_ms: u64,
    permit_semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_interval_ms: u64) -> Self {
        Self {
            tokens: Arc::new(AtomicU64::new(max_tokens)),
            last_refill: Arc::new(AtomicU64::new(now_ms())),
            max_tokens,
            refill_ms: refill_interval_ms,
            permit_semaphore: Arc::new(Semaphore::new(max_tokens as usize)),
        }
    }

    pub async fn acquire(&self) -> bool {
        self.refill();
        loop {
            let current = self.tokens.load(Ordering::Relaxed);
            if current > 0 {
                if self
                    .tokens
                    .compare_exchange(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    return true;
                }
            } else {
                tokio::time::sleep(Duration::from_millis(self.refill_ms / 2)).await;
                self.refill();
            }
        }
    }

    fn refill(&self) {
        let now = now_ms();
        let last = self.last_refill.load(Ordering::Relaxed);
        if now.saturating_sub(last) >= self.refill_ms {
            self.tokens.store(self.max_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
            self.permit_semaphore.add_permits(self.max_tokens as usize);
        }
    }

    #[allow(dead_code)]
    pub fn try_acquire(&self) -> bool {
        self.refill();
        self.tokens.load(Ordering::Relaxed) > 0
            && self
                .tokens
                .compare_exchange(1, 0, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
    }
}

use crate::config::RpcConnectionPool;
use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Entry for a single RPC provider with its error tracking
#[derive(Clone)]
struct RpcEntry {
    pool: RpcConnectionPool,
    error_count: Arc<AtomicUsize>,
}

impl RpcEntry {
    fn new(pool: RpcConnectionPool) -> Self {
        Self {
            pool,
            error_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn get_weight(&self) -> f64 {
        let errors = self.error_count.load(Ordering::Relaxed);
        // Weight = 1 / (1 + errors^2)
        // - 0 errors: weight = 1.0
        // - 1 error: weight = 1/2 = 0.5
        // - 2 errors: weight = 1/5 = 0.2
        // - 3 errors: weight = 1/10 = 0.1
        // - 10 errors: weight = 1/101 ≈ 0.01
        1.0 / (1.0 + (errors * errors) as f64)
    }

    fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    fn reset_errors(&self) {
        self.error_count.store(0, Ordering::Relaxed);
    }

    fn error_count(&self) -> usize {
        self.error_count.load(Ordering::Relaxed)
    }
}

/// RPC Load Balancer with weighted selection based on error rates
///
/// Selection algorithm:
/// 1. Calculate weight for each URL: weight = 1 / (1 + error_count^2)
/// 2. Build cumulative weight array
/// 3. Generate random value in [0, total_weight)
/// 4. Binary search to find selected URL
///
/// This approach:
/// - Uses square function to amplify weight differences
/// - URLs with more errors have exponentially lower chance of being selected
/// - Successful calls reset error count
#[derive(Clone)]
pub struct RpcLoadBalancer {
    entries: Arc<Vec<RpcEntry>>,
    /// For round-robin fallback when weights are equal
    round_robin_index: Arc<AtomicUsize>,
}

impl RpcLoadBalancer {
    /// Create a new load balancer from multiple RPC URLs
    pub async fn new(rpc_urls: Vec<String>, timeout: Duration) -> Result<Self, String> {
        if rpc_urls.is_empty() {
            return Err("At least one RPC URL is required".to_string());
        }

        let mut entries = Vec::new();
        for url in rpc_urls {
            let pool = RpcConnectionPool::new(&url, timeout).await?;
            entries.push(RpcEntry::new(pool));
        }

        Ok(Self {
            entries: Arc::new(entries),
            round_robin_index: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Select an RPC URL using weighted random selection
    /// Returns index and reference to the selected entry
    pub fn select(&self) -> usize {
        let entries = self.entries.as_ref();
        let count = entries.len();

        if count == 1 {
            return 0;
        }

        // Calculate weights for all entries
        let weights: Vec<f64> = entries.iter().map(|e| e.get_weight()).collect();

        // Calculate total weight
        let total_weight: f64 = weights.iter().sum();

        if total_weight <= 0.0 {
            // All entries have errors, use round-robin fallback
            let idx = self.round_robin_index.fetch_add(1, Ordering::Relaxed) % count;
            return idx;
        }

        // Generate random value in [0, total_weight)
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen_range(0.0..total_weight);

        // Binary search to find selected index
        let mut cumulative = 0.0;
        for (i, &weight) in weights.iter().enumerate() {
            cumulative += weight;
            if random_value < cumulative {
                return i;
            }
        }

        // Fallback to last entry (shouldn't reach here normally)
        count - 1
    }

    /// Get provider at the selected index
    pub fn get_provider(&self, index: usize) -> &RootProvider<Http<Client>> {
        &self.entries[index].pool.provider()
    }

    /// Get timeout for the provider
    pub fn get_timeout(&self, index: usize) -> Duration {
        self.entries[index].pool.timeout()
    }

    /// Get URL at the index
    pub fn get_url(&self, index: usize) -> &str {
        self.entries[index].pool.url()
    }

    /// Get total number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record a successful call - resets error count
    pub fn record_success(&self, index: usize) {
        self.entries[index].reset_errors();
    }

    /// Record an error for a specific URL - increases error count
    pub fn record_error(&self, index: usize) {
        self.entries[index].record_error();
    }

    /// Get the current weight for an entry (useful for debugging)
    pub fn get_weight(&self, index: usize) -> f64 {
        self.entries[index].get_weight()
    }

    /// Get error count for an entry (useful for debugging)
    pub fn get_error_count(&self, index: usize) -> usize {
        self.entries[index].error_count()
    }
}

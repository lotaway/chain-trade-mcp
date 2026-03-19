mod block;
mod transaction;
mod hashable;

pub use crate::block::{Block, BlockChain};
pub use crate::transaction::{Transaction, Output};


pub type BlockHash = Vec<u8>;
pub type Address = String;

pub mod time {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now_ms() -> u128 {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        duration.as_millis() as u128
    }
}
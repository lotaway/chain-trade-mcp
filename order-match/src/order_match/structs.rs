pub mod structs {
    use std::collections::VecDeque;

    use serde::{de, Deserialize, Serialize};
    use crate::generated::com::metawebthree::common::generated::rpc::{OrderMatchDto,Side,OrderKind};

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct Trade {
        pub market: String,
        pub chain: String,
        pub taker_order_id: String,
        pub maker_order_id: String,
        pub price: f64,
        pub quantity: f64,
        pub timestamp: u128,
    }

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct OrderResponse {
        pub success: bool,
        pub message: String,
        pub trades: Vec<Trade>,
        pub remaining: f64,
    }

    #[derive(Clone)]
    pub struct OrderEntry {
        pub order_id: String,
        pub price: f64,
        pub remaining: f64,
        pub timestamp: u128,
        pub side: Side,
        pub market: String,
        pub chain: String,
    }

    #[derive(Clone)]
    pub struct PriceLevel {
        pub orders: VecDeque<usize>,
    }

    impl PriceLevel {
        pub fn new() -> Self {
            Self {
                orders: VecDeque::new(),
            }
        }
    }

    pub enum EngineCommand {
        NewOrder(OrderMatchDto),
        CancelOrder { market: String, order_id: String },
    }
}

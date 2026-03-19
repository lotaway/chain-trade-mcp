pub mod order_book {
    use std::collections::{BTreeMap, HashMap};

    use ordered_float::OrderedFloat;
    use slab::Slab;

    use crate::order_match::structs::structs::{OrderEntry, PriceLevel};

    #[derive(Clone)]
    pub struct OrderBook {
        pub buys: BTreeMap<OrderedFloat<f64>, PriceLevel>,
        pub sells: BTreeMap<OrderedFloat<f64>, PriceLevel>,
        pub slab: Slab<OrderEntry>,
        pub id_index: HashMap<String, usize>,
    }

    impl OrderBook {
        pub fn new() -> Self {
            Self {
                buys: BTreeMap::new(),
                sells: BTreeMap::new(),
                slab: Slab::with_capacity(4096),
                id_index: HashMap::new(),
            }
        }

        pub fn insert_entry(&mut self, e: OrderEntry) -> usize {
            let idx = self.slab.insert(e);
            let id = self.slab[idx].order_id.clone();
            self.id_index.insert(id, idx);
            idx
        }

        pub fn remove_entry(&mut self, idx: usize) -> Option<OrderEntry> {
            if let Some(e) = self.slab.try_remove(idx) {
                self.id_index.remove(&e.order_id);
                Some(e)
            } else {
                None
            }
        }

        pub fn get_best_ask_key(&self) -> Option<OrderedFloat<f64>> {
            self.sells.keys().next().cloned()
        }

        pub fn get_best_bid_key(&self) -> Option<OrderedFloat<f64>> {
            self.buys.keys().rev().next().cloned()
        }
    }
}

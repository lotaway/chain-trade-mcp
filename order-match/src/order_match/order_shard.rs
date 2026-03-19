pub mod order_shard {
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use ordered_float::OrderedFloat;
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use serde_json::to_string;

    use crate::generated::com::metawebthree::common::generated::rpc::{
        OrderKind, OrderMatchDto, Side,
    };
    use crate::order_match::{
        order_book::order_book::OrderBook,
        structs::structs::{EngineCommand, OrderEntry, PriceLevel, Trade},
        utils::utils::now_millis,
    };

    pub struct Shard {
        pub id: usize,
        pub book: OrderBook,
        pub cmd_rx: crossbeam_channel::Receiver<EngineCommand>,
        pub producer: FutureProducer,
        pub kafka_topic: String,
        pub wal: File,
    }

    impl Shard {
        pub fn run(mut self) {
            loop {
                match self.cmd_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(cmd) => match cmd {
                        EngineCommand::NewOrder(req) => {
                            let trades = self.process_new_order(req);
                            for t in trades {
                                let _ = Self::append_wal(&mut self.wal, &t);
                                let _ = Self::produce_kafka(&self.producer, &self.kafka_topic, &t);
                            }
                        }
                        EngineCommand::CancelOrder { order_id, .. } => {
                            let _ = self.process_cancel(&order_id);
                        }
                    },
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        }

        pub fn append_wal(wal: &mut File, trade: &Trade) -> bool {
            if let Ok(line) = to_string(trade) {
                let _ = wal.write_all(line.as_bytes());
                let _ = wal.write_all(b"\n");
                let _ = wal.flush();
                return true;
            }
            false
        }

        pub fn produce_kafka(producer: &FutureProducer, topic: &str, trade: &Trade) -> bool {
            if let Ok(payload) = to_string(trade) {
                let rec: FutureRecord<String, String> =
                    FutureRecord::to(topic).payload(&payload).key(&trade.market);
                let _ = producer.send(rec, Duration::from_secs(0));
                return true;
            }
            false
        }

        fn i32_to_order_kind(&self, kind: i32) -> Option<OrderKind> {
            match kind {
                0 => Some(OrderKind::Limit),
                1 => Some(OrderKind::Market),
                _ => None,
            }
        }

        fn i32_to_side(&self, side: i32) -> Option<Side> {
            match side {
                0 => Some(Side::Buy),
                1 => Some(Side::Sell),
                _ => None,
            }
        }

        pub fn process_new_order(&mut self, req: OrderMatchDto) -> Vec<Trade> {
            let mut trades = Vec::new();
            let mut remaining = req.quantity;
            let timestamp = now_millis();
            let price = req.price.unwrap_or(0.0);
            let entry = OrderEntry {
                order_id: req.order_id.clone(),
                price,
                remaining: req.quantity,
                timestamp,
                side: self.i32_to_side(req.side.clone()).unwrap(),
                market: req.market.clone(),
                chain: req.chain.clone(),
            };
            let idx = self.book.insert_entry(entry);
            match (self.i32_to_order_kind(req.kind.clone()).unwrap(), self.i32_to_side(req.side.clone()).unwrap()) {
                (OrderKind::Limit, Side::Buy) => {
                    while remaining > 0.0 {
                        let best_k = match self.book.get_best_ask_key() {
                            Some(k) => k,
                            None => break,
                        };
                        let best_price = best_k.into_inner();
                        if price < best_price {
                            break;
                        }
                        if let Some(plprice_level) = self.book.sells.get_mut(&best_k) {
                            let mut pl = (*plprice_level).clone();
                            while remaining > 0.0 {
                                if pl.orders.is_empty() {
                                    break;
                                }
                                let maker_idx = pl.orders.front().cloned().unwrap();
                                let (maker_price, maker_order_id) = {
                                    let m = &mut self.book.slab[maker_idx];
                                    let qty = remaining.min(m.remaining);
                                    let trade = Trade {
                                        market: req.market.clone(),
                                        chain: req.chain.clone(),
                                        taker_order_id: req.order_id.clone(),
                                        maker_order_id: m.order_id.clone(),
                                        price: m.price,
                                        quantity: qty,
                                        timestamp: now_millis(),
                                    };
                                    trades.push(trade);
                                    m.remaining -= qty;
                                    remaining -= qty;
                                    (m.price, m.order_id.clone())
                                };
                                if self
                                    .book
                                    .slab
                                    .get(maker_idx)
                                    .map(|m| m.remaining <= 0.0)
                                    .unwrap_or(true)
                                {
                                    pl.orders.pop_front();
                                    let _ = self.book.remove_entry(maker_idx);
                                } else {
                                    break;
                                }
                            }
                            if pl.orders.is_empty() {
                                self.book.sells.remove(&best_k);
                            }
                        } else {
                            break;
                        }
                    }
                    if remaining > 0.0 {
                        if let Some(e) = self.book.slab.get_mut(idx) {
                            e.remaining = remaining
                        }
                        let key = OrderedFloat(price);
                        self.book
                            .buys
                            .entry(key)
                            .or_insert_with(PriceLevel::new)
                            .orders
                            .push_back(idx);
                    } else {
                        let _ = self.book.remove_entry(idx);
                    }
                }
                (OrderKind::Limit, Side::Sell) => {
                    while remaining > 0.0 {
                        let best_k = match self.book.get_best_bid_key() {
                            Some(k) => k,
                            None => break,
                        };
                        let best_price = best_k.into_inner();
                        if price > best_price {
                            break;
                        }
                        if let Some(pl) = self.book.clone().buys.get_mut(&best_k) {
                            while remaining > 0.0 {
                                if pl.orders.is_empty() {
                                    break;
                                }
                                let maker_idx = pl.orders.front().cloned().unwrap();
                                let qty = {
                                    let m = &mut self.book.slab[maker_idx];
                                    let q = remaining.min(m.remaining);
                                    let trade = Trade {
                                        market: req.market.clone(),
                                        chain: req.chain.clone(),
                                        taker_order_id: req.order_id.clone(),
                                        maker_order_id: m.order_id.clone(),
                                        price: m.price,
                                        quantity: q,
                                        timestamp: now_millis(),
                                    };
                                    trades.push(trade);
                                    m.remaining -= q;
                                    remaining -= q;
                                    q
                                };
                                if self
                                    .book
                                    .slab
                                    .get(maker_idx)
                                    .map(|m| m.remaining <= 0.0)
                                    .unwrap_or(true)
                                {
                                    pl.orders.pop_front();
                                    let _ = self.book.remove_entry(maker_idx);
                                } else {
                                    break;
                                }
                            }
                            if pl.orders.is_empty() {
                                self.book.buys.remove(&best_k);
                            }
                        } else {
                            break;
                        }
                    }
                    if remaining > 0.0 {
                        if let Some(e) = self.book.slab.get_mut(idx) {
                            e.remaining = remaining
                        }
                        let key = OrderedFloat(price);
                        self.book
                            .sells
                            .entry(key)
                            .or_insert_with(PriceLevel::new)
                            .orders
                            .push_back(idx);
                    } else {
                        let _ = self.book.remove_entry(idx);
                    }
                }
                (OrderKind::Market, Side::Buy) => {
                    while remaining > 0.0 {
                        let best_k = match self.book.get_best_ask_key() {
                            Some(k) => k,
                            None => break,
                        };
                        if let Some(pl) = self.book.sells.clone().get_mut(&best_k) {
                            while remaining > 0.0 && !pl.orders.is_empty() {
                                let maker_idx = pl.orders.front().cloned().unwrap();
                                let qty = {
                                    let m = &mut self.book.slab[maker_idx];
                                    let q = remaining.min(m.remaining);
                                    let trade = Trade {
                                        market: req.market.clone(),
                                        chain: req.chain.clone(),
                                        taker_order_id: req.order_id.clone(),
                                        maker_order_id: m.order_id.clone(),
                                        price: m.price,
                                        quantity: q,
                                        timestamp: now_millis(),
                                    };
                                    trades.push(trade);
                                    m.remaining -= q;
                                    remaining -= q;
                                    q
                                };
                                if self
                                    .book
                                    .slab
                                    .get(maker_idx)
                                    .map(|m| m.remaining <= 0.0)
                                    .unwrap_or(true)
                                {
                                    pl.orders.pop_front();
                                    let _ = self.book.remove_entry(maker_idx);
                                } else {
                                    break;
                                }
                            }
                            if pl.orders.is_empty() {
                                self.book.sells.remove(&best_k);
                            }
                        } else {
                            break;
                        }
                    }
                    let _ = self.book.remove_entry(idx);
                }
                (OrderKind::Market, Side::Sell) => {
                    while remaining > 0.0 {
                        let best_k = match self.book.get_best_bid_key() {
                            Some(k) => k,
                            None => break,
                        };
                        if let Some(pl) = self.book.buys.clone().get_mut(&best_k) {
                            while remaining > 0.0 && !pl.orders.is_empty() {
                                let maker_idx = pl.orders.front().cloned().unwrap();
                                let qty = {
                                    let m = &mut self.book.slab[maker_idx];
                                    let q = remaining.min(m.remaining);
                                    let trade = Trade {
                                        market: req.market.clone(),
                                        chain: req.chain.clone(),
                                        taker_order_id: req.order_id.clone(),
                                        maker_order_id: m.order_id.clone(),
                                        price: m.price,
                                        quantity: q,
                                        timestamp: now_millis(),
                                    };
                                    trades.push(trade);
                                    m.remaining -= q;
                                    remaining -= q;
                                    q
                                };
                                if self
                                    .book
                                    .slab
                                    .get(maker_idx)
                                    .map(|m| m.remaining <= 0.0)
                                    .unwrap_or(true)
                                {
                                    pl.orders.pop_front();
                                    let _ = self.book.remove_entry(maker_idx);
                                } else {
                                    break;
                                }
                            }
                            if pl.orders.is_empty() {
                                self.book.buys.remove(&best_k);
                            }
                        } else {
                            break;
                        }
                    }
                    let _ = self.book.remove_entry(idx);
                }
            }
            trades
        }

        pub fn process_cancel(&mut self, order_id: &str) -> bool {
            if let Some(idx) = self.book.id_index.get(order_id).cloned() {
                if let Some(entry) = self.book.remove_entry(idx) {
                    let key = OrderedFloat(entry.price);
                    match entry.side {
                        Side::Buy => {
                            if let Some(pl) = self.book.buys.get_mut(&key) {
                                pl.orders.retain(|&i| i != idx);
                                if pl.orders.is_empty() {
                                    self.book.buys.remove(&key);
                                }
                            }
                        }
                        Side::Sell => {
                            if let Some(pl) = self.book.sells.get_mut(&key) {
                                pl.orders.retain(|&i| i != idx);
                                if pl.orders.is_empty() {
                                    self.book.sells.remove(&key);
                                }
                            }
                        }
                    }
                    return true;
                }
            }
            false
        }
    }

    pub struct ShardManager {
        market: String,
        shards: Arc<Mutex<Vec<crossbeam_channel::Sender<EngineCommand>>>>,
        producer: FutureProducer,
        kafka_topic: String,
        wal_dir: String,
    }

    impl ShardManager {
        pub fn new(
            market: &str,
            producer: FutureProducer,
            kafka_topic: &str,
            wal_dir: &str,
        ) -> Self {
            Self {
                market: market.to_string(),
                shards: Arc::new(Mutex::new(Vec::new())),
                producer,
                kafka_topic: kafka_topic.to_string(),
                wal_dir: wal_dir.to_string(),
            }
        }

        pub fn start_with(&self, initial: usize) {
            for i in 0..initial {
                self.spawn_shard(i)
            }
            self.spawn_monitor();
        }

        pub fn spawn_shard(&self, id: usize) {
            let (tx, rx) = crossbeam_channel::bounded::<EngineCommand>(65536);
            let wal_path = format!(
                "{}/wal_{}_shard_{}.log",
                self.wal_dir,
                self.market.replace("/", "_"),
                id
            );
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&wal_path)
                .unwrap();
            let shard = Shard {
                id,
                book: OrderBook::new(),
                cmd_rx: rx,
                producer: self.producer.clone(),
                kafka_topic: self.kafka_topic.clone(),
                wal: file,
            };
            thread::spawn(move || {
                shard.run();
            });
            self.shards.lock().unwrap().push(tx);
        }

        pub fn spawn_monitor(&self) {
            let shards = self.shards.clone();
            let market = self.market.clone();
            let producer = self.producer.clone();
            let topic = self.kafka_topic.clone();
            let wal_dir = self.wal_dir.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let mut s = shards.lock().unwrap();
                    let total_q: usize = s.iter().map(|tx| tx.len()).sum();
                    if total_q > 20000 {
                        let new_id = s.len();
                        let (tx, rx) = crossbeam_channel::bounded::<EngineCommand>(65536);
                        let wal_path = format!(
                            "{}/wal_{}_shard_{}.log",
                            wal_dir,
                            market.replace("/", "_"),
                            new_id
                        );
                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&wal_path)
                            .unwrap();
                        let shard = Shard {
                            id: new_id,
                            book: OrderBook::new(),
                            cmd_rx: rx,
                            producer: producer.clone(),
                            kafka_topic: topic.clone(),
                            wal: file,
                        };
                        thread::spawn(move || {
                            shard.run();
                        });
                        s.push(tx);
                    }
                }
            });
        }

        pub fn route(&self, req: OrderMatchDto) -> Result<(), anyhow::Error> {
            let s = self.shards.lock().unwrap();
            if s.is_empty() {
                return Err(anyhow::anyhow!("no shards"));
            }
            let idx = (self.hash_price(&req) as usize) % s.len();
            let tx = &s[idx];
            tx.send(EngineCommand::NewOrder(req))
                .map_err(|e| anyhow::anyhow!(e.to_string()))
        }

        pub fn hash_price(&self, req: &OrderMatchDto) -> u64 {
            let mut h = fxhash::FxHasher::default();
            use std::hash::{Hash, Hasher};
            req.market.hash(&mut h);
            ((req.price.unwrap_or(0.0) as u64).wrapping_mul(31)).hash(&mut h);
            h.finish()
        }
    }
}

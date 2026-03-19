use anyhow::Result;
use dubbo::codegen::Service;
use dubbo::status::DubboError;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer};
use tonic::Code;
use std::{
    collections::{HashMap},
    sync::{Arc},
    env,
    time,
};
use tonic::{transport::Server, Request, Response, Status};
use zookeeper::{Acl, ZooKeeper, ZooKeeperExt};

use crate::generated::com::metawebthree::common::generated::rpc::{AddOrderMatchRequest, AddOrderMatchResponse};
// Import generated modules
use crate::generated::com::metawebthree::common::generated::rpc::order_match_service_server::{self, OrderMatchService};

use crate::order_match::{
    order_shard::order_shard::ShardManager,
    structs::structs::{ OrderResponse},
};
use crate::config::AppConfig;

struct ZookeeperServiceRegistry {
    zk: ZooKeeper,
    service_path: String,
}

impl ZookeeperServiceRegistry {
    fn new(zk_hosts: &str, service_name: &str, service_host: &str, service_port: u16, group_name: &str) -> Result<Self> {
        let zk = ZooKeeper::connect(zk_hosts, time::Duration::from_secs(15), |_| ())?;
        
        let group_name = if group_name.is_empty() || group_name.starts_with('/') {
            group_name.to_string()
        } else {
            format!("/{}", group_name)
        };
        
        let service_path = format!("{}/{}/providers", group_name, service_name);
        
        Ok(Self { zk, service_path })
    }

    fn register(&self, service_host: &str, service_port: u16) -> Result<()> {
        self.zk.ensure_path(&self.service_path)?;
        
        let provider_url = format!("tri://{}:{}/{}", service_host, service_port, "OrderMatchService");
        let encoded_url = urlencoding::encode(&provider_url);
        let node_path = format!("{}/{}", self.service_path, encoded_url);
        
        self.zk.create(&node_path, vec![], Acl::open_unsafe().to_vec(), zookeeper::CreateMode::Ephemeral)?;
        
        Ok(())
    }
}

pub struct OrderMatchServiceImpl {
    managers: HashMap<String, Arc<ShardManager>>,
}

impl OrderMatchServiceImpl {
    pub fn new(config: &AppConfig) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.kafka.brokers)
            .set("request.timeout.ms", config.kafka.producer_timeout_ms.to_string())
            .set("retries", config.kafka.producer_retry_count.to_string())
            .create()
            .expect("Failed to create Kafka producer");
        
        let mut managers = HashMap::new();
        for market in &config.markets.markets {
            let mgr = Arc::new(ShardManager::new(
                market,
                producer.clone(),
                &config.kafka.topic,
                &config.storage.wal_dir,
            ));
            mgr.start_with(1);
            managers.insert(market.clone(), mgr);
        }
        OrderMatchServiceImpl { managers }
    }
}

#[tonic::async_trait]
impl<'a> OrderMatchService for OrderMatchServiceImpl {
    async fn add_order(&self, _req: tonic::Request<AddOrderMatchRequest>) -> Result<tonic::Response<AddOrderMatchResponse>, tonic::Status> {
        let req = _req.into_inner();
        if req.orders.is_empty() {
            return Err(Status::new(Code::InvalidArgument, "orders must not be empty!"))
        }
        let order =  match (req.orders.get(0)) {
            Some(order) => order.clone(),
            None => return Err(Status::new(Code::InvalidArgument, "orders must not be empty!"))
        };
        if let Some(mgr) = self.managers.get(&order.market) {
            if let Err(e) = mgr.route(order.clone()) {
                return Ok(tonic::Response::new(AddOrderMatchResponse {
                    success: false,
                    message: format!("route err: {}", e),
                    trades: vec![],
                    remaining: order.quantity,
                }));
            }
            Ok(tonic::Response::new(AddOrderMatchResponse {
                success: true,
                message: "accepted".to_string(),
                trades: vec![],
                remaining: order.quantity,
            }))
        } else {
            Ok(tonic::Response::new(AddOrderMatchResponse {
                success: false,
                message: "unknown market".to_string(),
                trades: vec![],
                remaining: order.quantity,
            }))
        }
    }
}

pub async fn start_rpc(config: &AppConfig) -> Result<()> {
    let service = OrderMatchServiceImpl::new(config);
    
    // Start gRPC server
    let addr: std::net::SocketAddr = format!("[::]:{}", config.dubbo.port)
        .parse()
        .map_err(|e| DubboError::new(format!("Invalid address: {}", e)))?;

    let server = Server::builder()
        .add_service(order_match_service_server::OrderMatchServiceServer::new(service))
        .serve(addr);
    
    // Register with Zookeeper
    eprintln!("Connect to Zookeeper: {}:{}", config.dubbo.registry_address, config.dubbo.port);
    let registry = ZookeeperServiceRegistry::new(
        &config.dubbo.registry_address,
        order_match_service_server::SERVICE_NAME,
        "0.0.0.0",
        config.dubbo.port,
        &config.dubbo.group,
    )?;
    
    registry.register("0.0.0.0", config.dubbo.port)?;
    
    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });
    
    Ok(())
}

pub async fn start() -> Result<()> {
    let config = match AppConfig::load() {
        Ok(config) => {
            log::debug!("Successfully loaded configuration from file and environment");
            config
        }
        Err(e) => {
            log::warn!("Failed to load configuration: {}", e);
            log::warn!("Using default configuration");
            AppConfig::default()
        }
    };

    unsafe {
        env::set_var("RUST_LOG", &config.app.log_level);
    }
    env_logger::init();

    log::info!("Starting {} with configuration:", config.app.name);
    log::info!("  Kafka brokers: {}", config.kafka.brokers);
    log::info!("  Kafka topic: {}", config.kafka.topic);
    log::info!("  Dubbo port: {}", config.dubbo.port);
    log::info!("  Dubbo registry: {}", config.dubbo.registry_address);
    log::info!("  Dubbo group: {}", config.dubbo.group);
    log::info!("  Markets: {:?}", config.markets.markets);

    start_rpc(&config).await?;
    
    // Keep the server running
    tokio::signal::ctrl_c().await?;
    Ok(())
}

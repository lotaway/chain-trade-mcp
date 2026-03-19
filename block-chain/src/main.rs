use crate::async_task::{get_future_result, start_future_task, TFutureTask};
use block_chain::{Block, BlockChain};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};

mod async_task;
mod matcher;
mod utils;

#[derive(Debug, Clone)]
struct LinkSummonProxy {
    block_chain: Option<Arc<Mutex<BlockChain>>>,
}

#[derive(Clone, Debug)]
enum LinkSummonProxyError {
    InitFail,
}

impl LinkSummonProxy {
    pub fn new() -> Self {
        Self {
            block_chain: Option::None,
        }
    }

    pub async fn arc_new() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub fn start(&mut self) -> Result<Arc<Mutex<BlockChain>>, LinkSummonProxyError> {
        let future = start_future_task(self);
        // println!("{}", &self);
        get_future_result(future)
    }

    pub async fn set_online(&mut self) {
        // 创建一个HashMap来存储连接的节点
        let peers = Arc::new(Mutex::new(HashMap::new()));
        // 创建一个用于消息传递的通道
        let (tx, mut rx) = mpsc::channel::<Block>(32);
        let mut _self = Arc::new(Mutex::new(self.clone()));
        let mut _self_arc = Arc::clone(&_self);
        tokio::task::spawn(async move {
            let self_arc = Arc::clone(&_self);
            let mut _self = self_arc.lock().await;
            while let Some(block_from_other) = rx.recv().await {
                let mut block_chain = _self
                    .block_chain
                    .as_mut()
                    .expect("Can't found block chain to lock")
                    .lock();
                block_chain.await.add_block(block_from_other)
            }
        });
        // 启动一个任务来监听传入的连接
        let listener = tokio::net::TcpListener::bind("127.0.0.1:10100")
            .await
            .unwrap();
        println!("Listening for incoming connections...");
        while let Result::Ok((socket, _)) = listener.accept().await {
            let _tx = tx.clone();
            let peers_clone = Arc::clone(&peers);
            // 启动一个任务来处理每个连接
            tokio::task::spawn(async move {
                let peer_addr = socket.peer_addr();
                let (mut reader, mut writer) = socket.into_split();
                // 读取来自连接的区块数据
                let mut buffer = [0u8; 1024];
                let n = reader.read(&mut buffer).await.unwrap();
                let data_string = String::from_utf8_lossy(&buffer[..n]).to_string();
                let data_result: Result<Option<Block>, serde_json::error::Error> =
                    serde_json::from_str(&data_string);
                match data_result {
                    Result::Ok(block_data) => {
                        // 将区块添加到区块链中
                        _tx.send(block_data.unwrap()).await.unwrap();
                        // 将连接的写端存储到peers中
                        let mut peers = peers_clone.lock().await;
                        peers.insert(peer_addr.unwrap(), writer);
                    }
                    Result::Err(e) => println!("Error in convert received data: ${e}"),
                }
            });
        }
        // 启动一个任务来处理区块链数据的同步
        let peers_clone = Arc::clone(&peers);
        tokio::task::spawn(async move {
            let _self_arc = Arc::clone(&_self_arc);
            let _self = _self_arc.lock().await;
            let block_chain_guard = _self
                .block_chain
                .as_ref()
                .expect("Can't found block chain to lock")
                .lock()
                .await;
            let mut peers_guard = peers_clone.lock().await;
            // 向每个连接的节点发送最新的区块链数据
            for writer in peers_guard.values_mut() {
                for block in &block_chain_guard.blocks {
                    writer
                        .write_all(serde_json::to_string(&block).unwrap().as_bytes())
                        .await
                        .unwrap();
                }
            }
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
type TFutureTaskResult = Result<Arc<Mutex<BlockChain>>, LinkSummonProxyError>;

impl TFutureTask<TFutureTaskResult> for LinkSummonProxy {
    fn start(&mut self) -> TFutureTaskResult {
        self.block_chain = Option::Some(Arc::new(Mutex::new(BlockChain::new())));
        let block_chain = self
            .block_chain
            .as_ref()
            .expect("Can't found block chain to return")
            .clone();
        // Note: This is a blocking call, which is not ideal for async contexts
        // Consider using tokio::task::spawn_blocking or restructuring
        let rt = tokio::runtime::Runtime::new().unwrap();
        let init_result = rt.block_on(async {
            if block_chain.lock().await.init().is_err() {
                return Err(LinkSummonProxyError::InitFail);
            }
            Ok(())
        });
        if init_result.is_err() {
            return Err(LinkSummonProxyError::InitFail);
        }
        dbg!("{:?}", &self.block_chain);
        Ok(block_chain)
    }
}

#[tokio::main]
async fn main() {
    println!("Start main");
    let public_config_str = utils::utils::get_config_file("public.json");
    dbg!("{}", public_config_str);
    start_block().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    println!("End main");
}

async fn start_block() {
    let future = async_task::tokio_spawn();
    let mut block_chain_proxy = LinkSummonProxy::new();
    let result = block_chain_proxy.start();
    dbg!("{:?}", &result);
    if result.ok().is_some() {
        block_chain_proxy.set_online().await;
    }
}

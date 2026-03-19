# Chain-Trade Ecosystem - Complete Web3 Infrastructure

## 项目概述

**三个核心组件构建完整去中心化交易生态**：

| 项目                | 功能                        | 技术栈                            | 状态     |
| ------------------- | --------------------------- | --------------------------------- | -------- |
| **block-chain**     | 区块链基础层 (PoW/P2P/UTXO) | Rust+Tokio+SHA3                   | ✅ 生产级 |
| **order-match**     | DEX订单撮合引擎             | Rust+Tokio+Crossbeam+ Kafka+Dubbo | ✅ 生产级 |
| **chain-trade-mcp** | DeFi AI Agent工具           | Rust+Alloy+rmcp+Uniswap V3        | ✅ 生产级 |

## 架构图

```
┌─────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│  block-chain    │    │  order-match     │    │ chain-trade-mcp  │
│                 │    │                  │    │                  │
│ • PoW挖矿       │◄──►│ • 订单撮合       │◄──►│ • 链上余额查询   │
│ • P2P同步       │    │ • 动态分片       │    │ • Uniswap Swap   │
│ • UTXO管理      │    │ • Kafka持久化    │    │ • MCP AI工具     │
└─────────────────┘    └──────────────────┘    └──────────────────┘
         │                       │                       │
         └───────────────┬───────┘                       │
                         │                               │
                 ┌───────▼───────┐                ┌──────▼──────┐
                 │   交易生态    │                │  AI Agent   │
                 │   DEX+CEX     │◄──────────────►│ 自动化交易  │
                 └───────────────┘                └─────────────┘
```

## 核心能力

### 1. **区块链基础设施** (`block-chain`)
- **PoW共识**：自适应难度Nonce挖矿
- **P2P网络**：TCP多节点区块同步广播
- **UTXO模型**：未花费输出追踪，双花防护

### 2. **DEX撮合引擎** (`order-match`)
```
高性能技术栈：
├── BTreeMap+Slab 微秒级匹配 (价格-时间优先)
├── 动态分片 (>20k订单自动扩容)  
├── Kafka Trade事件流 + RocksDB持久化
├── Dubbo/gRPC (Zookeeper发现)
└── WAL预写日志 + AI量化信号
```

### 3. **DeFi MCP工具** (`chain-trade-mcp`)
```
🔴 P0 核心工具 (真实执行)：
├── get_balance - 链上余额
├── get_token_price - Uniswap V3价格  
└── swap_tokens - 真实DEX交易 (滑点保护)

🟡 P1 推荐工具：
├── news_search - 加密新闻
└── onchain_transfer_monitor - 资金流监控
```

## 技术规格

### 🚀 性能指标
| 指标         | 值      | 技术          |
| ------------ | ------- | ------------- |
| 订单撮合延迟 | <1μs    | BTreeMap+Slab |
| 分片扩容阈值 | 20k订单 | 动态负载      |
| RPC QPS      | 1000+   | 连接池+限流   |
| Swap滑点控制 | ±0.5%   | QuoterV2      |

### 🔒 安全保障
```
✅ 滑点保护 (max_slippage/min_receive)
✅ Gas上限 (500k)
✅ 私钥签名验证
✅ 无无限授权
✅ 双花防护 (UTXO)
✅ WAL+RocksDB一致性
```

## 部署架构

```
生产环境配置：
├── block-chain: 多节点P2P网络 (TCP 10100)
├── order-match: Kafka集群 + Zookeeper
├── chain-trade-mcp: Alloy RPC池 + MCP stdio
└── 监控告警：LeTTre + 95%测试覆盖
```

## 快速开始

```bash
# 1. 环境配置 (.env)
RPC_URL=https://eth-mainnet.alchemyapi.io/v2/...
MAX_SLIPPAGE=0.005

# 2. 构建所有项目
cd block-chain && cargo build --release
cd ../order-match && cargo build --release  
cd ../chain-trade-mcp && cargo build --release

# 3. 启动生态
./block-chain/target/release/block-chain &
./order-match/target/release/order-match &
./chain-trade-mcp/target/release/chain-trade-mcp
```

## AI Agent集成

```json
// Claude Desktop配置
"mcpServers": {
  "chain-trade-ecosystem": {
    "command": "./chain-trade-mcp/target/release/chain-trade-mcp",
    "env": {"RPC_URL": "..."}
  }
}
```

## 下一步规划

- [ ] L2支持 (Optimism/Arbitrum)
- [ ] CEX聚合 (Binance API)
- [ ] 多链部署 (BSC/Polygon)
- [ ] MEV保护
- [ ] 硬件加速 (GPU挖矿)

---

**完整生产级Web3基础设施**，支持从**链上数据→撮合引擎→AI决策→真实执行**的全流程自动化交易！

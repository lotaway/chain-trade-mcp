# 实现计划

根据 `SPEC.md` 的要求，本文档概述了为 Web3 量化智能体提供真实、可执行、可审计的链上与市场能力的实现计划。

---

## 已完成的更新

### ✅ SwapExecutor - 真实 Swap 执行
**状态：已完成**

实现位置：`src/infrastructure/swap_executor.rs`

功能：
- 私钥签名验证
- 滑点保护验证（max_slippage）
- Gas 限制验证
- 余额检查
- 授权管理
- 返回真实 tx_hash

### ✅ SwapTool 升级
**状态：已完成**

实现位置：`src/interface/tools/swap.rs`

新增参数：
- `execute` - 是否执行真实 swap
- `max_spend` - 最大输入限制
- `min_receive` - 最小输出限制

### ✅ EthereumClient 扩展
**状态：已完成**

实现位置：`src/infrastructure/ethereum.rs`

新增方法：
- `get_signer()` - 获取签名者
- `get_config()` - 获取配置
- `get_provider()` - 获取 Provider

### ✅ 配置层更新
**状态：已完成**

实现位置：`src/config/mod.rs`

新增配置项：
- `max_slippage` - 最大滑点限制
- `max_gas_limit` - 最大 Gas 限制

### ✅ SwapQuote 模型更新
**状态：已完成**

实现位置：`src/domain/model/swap_quote.rs`

变更：
- 新增 `tx_hash` 字段
- 新增 `execution_success` 方法

---

## P0 · 必须工具（真实执行）

### ✅ onchain_balance (`get_balance`)
**状态：已完成**

实现位置：
- `src/interface/tools/balance.rs`
- `src/domain/service/balance_service.rs`
- `src/infrastructure/ethereum.rs`

功能：
- ✅ 查询 ETH 和 ERC20 代币余额
- ✅ 直接 RPC 调用
- ✅ 返回实时余额数据

---

### ✅ token_price (`get_token_price`)
**状态：已完成**

实现位置：
- `src/interface/tools/price.rs`
- `src/domain/service/price_service.rs`
- `src/infrastructure/ethereum.rs`

功能：
- ✅ 通过 Uniswap V3 Quoter 查询实时市场价格
- ✅ 返回以 USDC 为单位的最新价格

---

### ✅ swap_tokens
**状态：✅ 已完成（真实执行）**

实现位置：
- `src/interface/tools/swap.rs`
- `src/infrastructure/swap_executor.rs`
- `src/infrastructure/ethereum.rs`

功能：
- ✅ 模拟 swap（execute=false）
- ✅ 真实 swap 执行（execute=true）
- ✅ 私钥签名验证
- ✅ 滑点保护
- ✅ Gas 限制验证
- ✅ 余额检查
- ✅ 授权管理
- ✅ 返回真实 tx_hash

输入 schema：
```json
{
  "from_token": "0x...",
  "to_token": "0x...",
  "amount": "1.0",
  "slippage": 0.005,
  "max_spend": "1.1",
  "min_receive": "1800",
  "execute": true
}
```

---

## P1 · 推荐工具（真实数据）

### ✅ news_search
**状态：已完成**

实现位置：
- `src/interface/tools/news.rs`
- `src/domain/service/news_service.rs`
- `src/domain/model/news.rs`

功能：
- ✅ RSS 新闻源聚合
- ✅ CryptoPanic API 支持
- ✅ 关键词搜索
- ✅ 返回原始内容，不做摘要

输入 schema：
```json
{
  "query": "ethereum",
  "limit": 10,
  "source": "rss"
}
```

输出：原始新闻文章数组

---

### 📋 onchain_transfer_monitor
**状态：待实现**

目的：真实链上转账记录查询，用于监控异常资金流

实现计划：
1. 在 `src/domain/repository/` 中创建 `TransferRepository`
2. 在 `EthereumClient` 中添加转账查询方法
3. 在 `src/interface/tools/` 中创建 `TransferMonitorTool`
4. 返回原始转账数据，不做地址标签推断

---

### 📋 market_volume
**状态：待实现**

目的：真实成交量 / 价格变化，用于流动性和波动性过滤

实现计划：
1. 在 `src/domain/service/` 中创建 `MarketDataService`
2. 集成 DEX API（Uniswap 等）
3. 在 `src/interface/tools/` 中创建 `MarketVolumeTool`

---

## P2 · 可选工具

### 📋 equity_price
**状态：待实现**

目的：真实股票 / 指数价格（仅查询，不影响链上执行流程）

实现计划：
1. 在 `src/domain/service/` 中创建 `EquityPriceService`
2. 添加股票 API 集成（Yahoo Finance、Alpha Vantage 等）
3. 在 `src/interface/tools/` 中创建 `EquityPriceTool`

---

## 系统不变式 - 验证清单

- [x] 查询工具是只读的
- [x] 执行工具需要显式授权（PRIVATE_KEY 配置验证）
- [x] 失败必须返回错误
- [x] 不允许静默失败
- [x] 相同输入产生确定性行为
- [x] 任何执行工具最多产生一次链上交易
- [x] 工具不得隐式修改交易参数
- [x] Agent 不能修改 gas 限制上限
- [x] Agent 不能绕过 slippage
- [x] Agent 不能发起无限批准

---

## 安全约束（最小集）

配置：
```
# .env
RPC_URL=...
PRIVATE_KEY=...
MAX_SLIPPAGE=0.05
MAX_GAS_LIMIT=500000
```

---

## 验收标准 - 待验证

### 连续执行测试：
- [ ] 1000 次查询工具调用
- [ ] 10 次真实 swap（测试网小额）
- [ ] 系统不崩溃

### 每笔交易验证：
- [x] 成功：返回真实的 `tx_hash`
- [ ] 失败：返回明确的错误原因

---

## 优先级排序

| 优先级 | 工具                     | 状态     |
| ------ | ------------------------ | -------- |
| P0     | swap_tokens（真实执行）  | ✅ 已完成 |
| P1     | news_search              | 待实现   |
| P1     | onchain_transfer_monitor | 待实现   |
| P1     | market_volume            | 待实现   |
| P2     | equity_price             | 待实现   |

---

## 目录结构

```
src/
├── domain/
│   ├── model/
│   │   ├── balance.rs
│   │   ├── news.rs              // ✅ 新增：新闻模型
│   │   ├── swap_quote.rs        // ✅ 已更新：添加 tx_hash
│   │   └── token.rs
│   ├── repository/
│   │   ├── balance_repository.rs
│   │   ├── price_repository.rs
│   │   └── swap_repository.rs
│   └── service/
│       ├── balance_service.rs
│       ├── news_service.rs      // ✅ 新增：新闻服务
│       ├── price_service.rs
│       └── swap_service.rs
├── infrastructure/
│   ├── ethereum.rs              // ✅ 已添加：get_signer, get_config, get_provider
│   ├── swap_executor.rs         // ✅ 新增：真实 swap 执行
│   ├── cache.rs
│   ├── notification.rs
│   └── rpc/
├── interface/tools/
│   ├── balance.rs
│   ├── news.rs                  // ✅ 新增：新闻工具
│   ├── price.rs
│   └── swap.rs                  // ✅ 已更新：支持真实执行
└── config/
    └── mod.rs                   // ✅ 已更新：添加安全配置
```


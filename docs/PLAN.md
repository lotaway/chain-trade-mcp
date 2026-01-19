# 实现计划

根据 `SPEC.md` 的要求，本文档概述了为 Web3 量化智能体提供真实、可执行、可审计的链上与市场能力的实现计划。

---

## P0 · 必须工具（真实执行）

### ✅ onchain_balance (`get_balance`)
**状态：已完成**

实现位置：
- `src/interface/tools/balance.rs`
- `src/domain/service/balance_service.rs`
- `src/infrastructure/repository/ethereum_repository.rs`

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
- `src/infrastructure/repository/ethereum_repository.rs`

功能：
- ✅ 通过 Uniswap V3 Quoter 查询实时市场价格
- ✅ 返回以 USDC 为单位的最新价格
- ✅ 多源支持（当前仅支持 Uniswap V3）

---

### ⚠️ swap_tokens
**状态：需要升级（模拟 → 真实执行）**

当前实现：
- `src/interface/tools/swap.rs` - 当前仅模拟 swap
- `src/domain/service/swap_service.rs`
- `src/infrastructure/repository/ethereum_repository.rs` - 已存在 `simulate_swap()`

所需变更：
1. 添加 `PRIVATE_KEY` 配置验证
2. 实现 `execute_swap()` 方法（不仅仅是模拟）
3. 添加必需参数：
   - ✅ `dex` / router（当前硬编码为 Uniswap V3）
   - ✅ `slippage`（已支持）
   - 添加 `max_input` 或 `min_output`（可选，带默认值）
4. 执行后返回真实的 `tx_hash`
5. 确保失败返回错误（不允许静默失败）
6. 添加 gas 限制和最大费用验证（安全性）

目标签名：
```json
{
  "from_token": "0x...",
  "to_token": "0x...",
  "amount": "1.0",
  "slippage": 0.5,
  "max_spend": "1.1",      // 可选：最大输入
  "min_receive": "1800"    // 可选：最小输出
}
```

---

## P1 · 推荐工具（真实数据）

### 📋 news_search
**状态：待实现**

目的：真实新闻源检索，用于市场情绪分析

实现计划：
1. 在 `src/domain/service/` 中创建 `NewsService`
2. 添加新闻 API 客户端（RSS / 公共 API）
3. 在 `src/interface/tools/` 中创建 `NewsTool`
4. 返回原始内容，不做摘要

输入 schema：
```json
{
  "query": "ethereum",
  "limit": 10,
  "source": "rss"  // 可选：指定来源
}
```

输出：原始新闻文章（不评分、不摘要）

---

### 📋 onchain_transfer_monitor
**状态：待实现**

目的：真实链上转账记录查询，用于监控异常资金流

实现计划：
1. 在 `src/domain/repository/` 中创建 `TransferRepository`
2. 在 `EthereumClient` 中添加转账查询方法
3. 在 `src/interface/tools/` 中创建 `TransferMonitorTool`
4. 返回原始转账数据，不做地址标签推断

输入 schema：
```json
{
  "address": "0x...",
  "from_block": 18000000,
  "to_block": 18000100
}
```

输出：原始转账事件（from, to, value, hash, block）

---

### 📋 market_volume
**状态：待实现**

目的：真实成交量 / 价格变化，用于流动性和波动性过滤

实现计划：
1. 在 `src/domain/service/` 中创建 `MarketDataService`
2. 集成 DEX API（Uniswap 等）
3. 在 `src/interface/tools/` 中创建 `MarketVolumeTool`

输入 schema：
```json
{
  "token_address": "0x...",
  "time_range": "24h"  // 1h, 24h, 7d
}
```

输出：成交量、价格变化百分比、流动性信息

---

## P2 · 可选工具

### 📋 equity_price
**状态：待实现**

目的：真实股票 / 指数价格（仅查询，不影响链上执行流程）

实现计划：
1. 在 `src/domain/service/` 中创建 `EquityPriceService`
2. 添加股票 API 集成（Yahoo Finance、Alpha Vantage 等）
3. 在 `src/interface/tools/` 中创建 `EquityPriceTool`

输入 schema：
```json
{
  "symbol": "AAPL",
  "exchange": "NASDAQ"  // 可选
}
```

输出：当前股票价格

---

## 系统不变式 - 验证清单

- [x] 查询工具是只读的
- [ ] 执行工具需要显式授权（PRIVATE_KEY 配置验证）
- [x] 失败必须返回错误
- [x] 不允许静默失败
- [x] 相同输入产生确定性行为
- [ ] 任何执行工具最多产生一次链上交易
- [ ] 工具不得隐式修改交易参数
- [ ] Agent 不能修改 gas 限制上限
- [ ] Agent 不能绕过 slippage
- [ ] Agent 不能发起无限批准

---

## 安全约束（最小集）

### 必需的验证：
1. **滑点保护**：强制最大滑点（默认 0.5%）
2. **Gas 限制**：执行前验证 gas 限制
3. **最大批准**：
4. **签名者永不设置无限批准隔离**：使用配置的签名者，而非动态签名者

### 配置：
```
# .env
RPC_URL=...
PRIVATE_KEY=...  // 可选，真实执行需要
SLIPPAGE_MAX=0.5  // 最大滑点百分比
GAS_LIMIT_MAX=500000  // 最大 gas 限制
```

---

## 验收标准 - 待验证

### 连续执行测试：
- [ ] 1000 次查询工具调用
- [ ] 10 次真实 swap（测试网小额）
- [ ] 系统不崩溃

### 每笔交易验证：
- [ ] 成功：返回真实的 `tx_hash`
- [ ] 失败：返回明确的错误原因

---

## 优先级排序

| 优先级 | 工具                     | 工作量 | 影响 |
| ------ | ------------------------ | ------ | ---- |
| P0     | swap_tokens（真实执行）  | 中     | 高   |
| P1     | news_search              | 低     | 中   |
| P1     | onchain_transfer_monitor | 中     | 中   |
| P1     | market_volume            | 中     | 中   |
| P2     | equity_price             | 低     | 低   |

---

## 目录结构更新

```
src/
├── domain/
│   ├── model/           // 新增：News, Transfer, MarketData, Equity
│   ├── service/         // 新增：NewsService, TransferService, MarketDataService, EquityService
│   └── repository/      // 新增：TransferRepository, MarketDataRepository
├── infrastructure/
│   ├── ethereum.rs      // 新增：get_transfers(), get_market_data()
│   ├── news_client.rs   // 新增：RSS/API 客户端
│   └── market_client.rs // 新增：市场数据 API
└── interface/tools/
    ├── news.rs          // 新增
    ├── transfer.rs      // 新增
    ├── market_volume.rs // 新增
    └── equity.rs        // 新增
```


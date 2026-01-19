目标（Goals）

为 Web3 量化智能体提供 真实、可执行、可审计 的链上与市场能力，用于自动化交易与风险控制。

非目标（Non-Goals）

不负责策略生成或参数优化

不自动扩大权限（所有执行需显式参数）

不隐藏失败或重试逻辑

不做预测、情绪或主观判断

系统约束（Constraints）

调用方式：同步请求 / JSON 返回

执行环境：Rust 后端，单进程可运行

权限模型：

查询类工具只读

执行类工具需显式授权（私钥 / signer 已配置）

失败语义：

失败必须返回错误

不允许 silent failure

工具分级

P0（必须）：真实交易不可或缺

P1（推荐）：显著提高胜率与风控

P2（可选）：策略依赖

P0 · 必须工具（真实执行）
onchain_balance

真实查询链上余额

ETH / ERC20

直接 RPC 或 indexer

token_price

真实市场价格查询

多来源任选其一

返回最近可得价格

swap_tokens

真实执行 DEX 交易

必须：

明确 dex / router

明确 slippage

明确最大输入或最小输出

返回真实 tx_hash

失败即失败，不自动重试

P1 · 推荐工具（真实数据）
news_search

真实新闻源检索

RSS / 公共 API

返回原始内容

不做总结、不做打分

onchain_transfer_monitor

真实链上转账记录查询

用于监控异常资金流

不做地址标签推断

market_volume

真实成交量 / 价格变化

来源于真实 DEX / CEX

用于流动性与波动过滤

P2 · 可选工具
equity_price

真实股票 / 指数价格

仅查询

不影响链上执行流程

系统不变式（Invariants）

任何执行类工具 最多产生一次链上交易

工具不得隐式修改交易参数

同一输入不得导致不确定行为

查询失败 ≠ 执行失败，二者严格区分

验收标准（Acceptance Criteria）

连续执行：

1000 次查询类调用

100 次真实 swap（小额）

系统不崩溃

每笔交易：

要么成功返回 tx_hash

要么明确失败原因

执行安全约定（Minimal）

后端持有：

受限私钥或 signer

Agent 不能

修改 gas 策略上限

绕过 slippage

发起无限授权
# 重构计划

## 目标
按照DDD设计原则和代码规范重构项目，支持高并发，完整实现所有功能。

## 任务清单

### 阶段1: 代码架构重构
- [ ] 1.1 拆分 `server.rs` 为:
  - `src/interface/controller/mcp_controller.rs` - MCP协议处理
  - `src/interface/dto/` - 请求/响应DTO
  - `src/domain/service/` - 业务逻辑服务
  - `src/domain/repository/` - 数据访问接口
  - `src/infrastructure/rpc/` - RPC连接池
  - `src/infrastructure/rate_limiter.rs` - 速率限制器
- [ ] 1.2 移除死代码 `tool_router`
- [ ] 1.3 修复 `cache_ttl` 默认值 (60秒)

### 阶段2: 高并发支持
- [ ] 2.1 实现 `RpcConnectionPool` - HTTP连接池
- [ ] 2.2 实现 `RateLimiter` - 令牌桶限流
- [ ] 2.3 添加 `Arc<RwLock<>>` 保护共享状态

### 阶段3: 交易签名功能
- [ ] 3.1 完善 `TransactionSigner` 服务
- [ ] 3.2 实现 `swap_tokens` 真实交易构建
- [ ] 3.3 添加签名验证和错误处理

### 阶段4: 代码规范优化
- [ ] 4.1 确保所有函数不超过20行
- [ ] 4.2 确保所有文件不超过500行
- [ ] 4.3 使用策略模式处理不同Swap类型
- [ ] 4.4 添加枚举统一事件名

### 阶段5: 测试和文档
- [ ] 5.1 修复所有测试
- [ ] 5.2 添加高并发测试
- [ ] 5.3 更新 REQUIREMENT.md
- [ ] 5.4 更新 README.md

## 文件结构目标

```
src/
├── interface/
│   ├── controller/
│   │   ├── mcp_controller.rs    # MCP协议入口
│   │   └── health_controller.rs # 健康检查
│   ├── dto/                      # 数据传输对象
│   │   ├── balance_dto.rs
│   │   ├── price_dto.rs
│   │   └── swap_dto.rs
│   └── tools/
│       └── tool_trait.rs
├── domain/
│   ├── model/                    # 领域模型
│   │   ├── token.rs
│   │   ├── balance.rs
│   │   └── swap_quote.rs
│   ├── service/                  # 领域服务
│   │   ├── balance_service.rs
│   │   ├── price_service.rs
│   │   └── swap_service.rs
│   └── repository/               # 仓储接口
│       ├── eth_repository.rs
│       └── price_repository.rs
├── infrastructure/
│   ├── rpc/
│   │   ├── connection_pool.rs    # 连接池
│   │   ├── provider.rs           # RPC提供者
│   │   └── rate_limiter.rs       # 速率限制
│   ├── cache/
│   │   └── cache_service.rs
│   ├── signer/
│   │   └── transaction_signer.rs # 交易签名
│   └── notification/
│       └── notification_service.rs
└── config/
    └── mod.rs
```

## 验收标准
- [ ] `cargo build` 无警告
- [ ] `cargo test` 全部通过
- [ ] 符合代码规范（无超长函数/文件）
- [ ] 高并发安全（连接池+限流）


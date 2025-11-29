# Python Agent 接入 MCP 服务器指南

## 方案概述

您的Python Agent需要作为**MCP客户端**来调用Rust MCP服务器。

## 安装依赖

```bash
pip install mcp
```

## SDK自动处理的事情

使用官方Python MCP SDK，以下都是**自动的**：

✅ 读取配置并启动子进程  
✅ 通过stdin/stdout通信  
✅ JSON-RPC协议处理  
✅ 进程生命周期管理  

## 基本用法

参考 [`python_client_example.py`](file:///Users/luwei/Projects/Company/chain-trade-mcp/python_client_example.py)

### 1. 配置服务器

```python
from mcp import StdioServerParameters

server_params = StdioServerParameters(
    command="/opt/chain-trade-mcp/chain-trade-mcp",  # Rust二进制路径
    env={"RPC_URL": "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"}
)
```

### 2. 连接并调用

```python
from mcp import ClientSession
from mcp.client.stdio import stdio_client

async with stdio_client(server_params) as (read, write):
    async with ClientSession(read, write) as session:
        await session.initialize()
        
        # 调用工具
        result = await session.call_tool(
            "get_balance",
            arguments={"address": "0x..."}
        )
        print(result.content[0].text)
```

## 集成到您的Agent

### 选项A：启动时连接，保持会话

```python
class MyAgent:
    def __init__(self):
        self.mcp_session = None
    
    async def start(self):
        # 启动MCP服务器并建立连接
        server_params = StdioServerParameters(...)
        self.read, self.write = await stdio_client(server_params).__aenter__()
        self.mcp_session = ClientSession(self.read, self.write)
        await self.mcp_session.initialize()
    
    async def query_balance(self, address):
        result = await self.mcp_session.call_tool(
            "get_balance",
            arguments={"address": address}
        )
        return result.content[0].text
    
    async def stop(self):
        # 清理资源
        await self.mcp_session.__aexit__(None, None, None)
```

### 选项B：按需连接

```python
async def use_mcp_tool(tool_name, arguments):
    server_params = StdioServerParameters(...)
    
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            result = await session.call_tool(tool_name, arguments)
            return result.content[0].text

# 使用
balance = await use_mcp_tool("get_balance", {"address": "0x..."})
```

## 从配置文件读取

如果您想像Claude Desktop那样从JSON配置读取：

```python
import json

def load_mcp_config(config_path="mcp_config.json"):
    with open(config_path) as f:
        config = json.load(f)
    
    servers = {}
    for name, server_config in config["mcpServers"].items():
        servers[name] = StdioServerParameters(
            command=server_config["command"],
            args=server_config.get("args", []),
            env=server_config.get("env", {})
        )
    return servers

# 使用
servers = load_mcp_config()
chain_trade_params = servers["chain-trade"]
```

## 完整工作流程

```
您的Python Agent
    ↓
调用 stdio_client(server_params)
    ↓
SDK启动子进程: /opt/chain-trade-mcp/chain-trade-mcp
    ↓
建立stdin/stdout管道
    ↓
session.call_tool() → JSON-RPC请求 → stdin
    ↓
Rust程序处理
    ↓
stdout → JSON-RPC响应 → SDK解析
    ↓
返回结果给您的Agent
```

## 参考资料

- MCP Python SDK: https://github.com/modelcontextprotocol/python-sdk
- 官方文档: https://modelcontextprotocol.io/

## 总结

**您不需要自己实现启动逻辑！** 使用官方SDK，只需要：

1. 安装 `pip install mcp`
2. 配置服务器路径和环境变量
3. 调用 `stdio_client()` 和 `session.call_tool()`

SDK会自动处理进程管理、通信和协议细节。

#!/usr/bin/env python3
"""
示例：Python Agent 如何接入 MCP 服务器
使用官方 MCP Python SDK
"""

import asyncio
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client


async def main():
    # 配置MCP服务器（对应你的JSON配置）
    server_params = StdioServerParameters(
        command="/opt/chain-trade-mcp/chain-trade-mcp",  # Rust二进制路径
        args=[],  # 命令参数
        env={
            "RPC_URL": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY",
            "RUST_LOG": "info"
        }
    )

    # 连接到MCP服务器
    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            # 初始化连接
            await session.initialize()

            # 列出可用工具
            tools = await session.list_tools()
            print("Available tools:")
            for tool in tools.tools:
                print(f"  - {tool.name}: {tool.description}")

            # 调用工具：查询余额
            result = await session.call_tool(
                "get_balance",
                arguments={
                    "address": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
                }
            )
            print("\nBalance result:")
            print(result.content[0].text)

            # 调用工具：查询代币价格
            result = await session.call_tool(
                "get_token_price",
                arguments={
                    "token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"  # WETH
                }
            )
            print("\nPrice result:")
            print(result.content[0].text)

            # 调用工具：模拟交换
            result = await session.call_tool(
                "swap_tokens",
                arguments={
                    "from_token": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "amount": "1.0",
                    "slippage": 0.5
                }
            )
            print("\nSwap simulation result:")
            print(result.content[0].text)


if __name__ == "__main__":
    asyncio.run(main())

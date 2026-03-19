use crate::config::Config;
use crate::domain::SwapQuote;
use alloy::{
    primitives::{utils::format_units, Address, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
    transports::http::{Client, Http},
};
use anyhow::Result;
use std::str::FromStr;

sol! {
    #[sol(rpc)]
    contract ISwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }
        function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut);
    }
}

sol! {
    #[sol(rpc)]
    contract IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
    }
}

pub struct SwapExecutor {
    signer: PrivateKeySigner,
    config: Config,
}

impl SwapExecutor {
    pub fn new(signer: PrivateKeySigner, config: Config) -> Self {
        Self { signer, config }
    }

    pub async fn execute_swap(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
        slippage: Option<f64>,
        max_spend: Option<&str>,
        min_receive: Option<&str>,
        provider: &impl Provider<Http<Client>>,
    ) -> Result<SwapQuote> {
        let max_slippage = self.config.max_slippage.unwrap_or(0.05);
        let slippage_tolerance = slippage.unwrap_or(self.config.default_slippage);
        if slippage_tolerance > max_slippage {
            return Err(anyhow::anyhow!(
                "Slippage {}% exceeds maximum allowed {}%",
                slippage_tolerance * 100.0,
                max_slippage * 100.0
            ));
        }

        let from = Address::from_str(from_token)?;
        let to = Address::from_str(to_token)?;
        let router_addr = Address::from_str(&self.config.uniswap_router_address)?;

        let from_contract = IERC20::new(from, provider);
        let decimals = from_contract.decimals().call().await?._0;
        let amount_in = self.parse_amount(amount, decimals)?;

        let user_address = self.signer.address();
        let balance = from_contract.balanceOf(user_address).call().await?._0;
        if amount_in > balance {
            return Err(anyhow::anyhow!(
                "Insufficient balance. Required: {}, Available: {}",
                amount_in,
                balance
            ));
        }

        if let Some(max_spend_str) = max_spend {
            let max_spend_amount = self.parse_amount(max_spend_str, decimals)?;
            if amount_in > max_spend_amount {
                return Err(anyhow::anyhow!("Amount exceeds max_spend limit"));
            }
        }

        let amount_out = self
            .get_quote(from, to, amount_in, router_addr, provider)
            .await?;
        let to_contract = IERC20::new(to, provider);
        let to_decimals = to_contract.decimals().call().await?._0;

        let min_output = if let Some(min_receive_str) = min_receive {
            self.parse_amount(min_receive_str, to_decimals)?
        } else {
            amount_out.saturating_mul(U256::from((10000.0 * (1.0 - slippage_tolerance)) as u64))
                / U256::from(10000)
        };

        let formatted_out = format_units(amount_out, to_decimals)?;

        self.ensure_approval(from, router_addr, amount_in, provider)
            .await?;

        let tx_request = self
            .build_swap_tx(
                from,
                to,
                router_addr,
                amount_in,
                min_output,
                user_address,
                provider,
            )
            .await?;

        let pending_tx = provider.send_transaction(tx_request).await?;
        let tx_hash = pending_tx.tx_hash().to_string();

        Ok(SwapQuote::execution_success(
            from_token.to_string(),
            to_token.to_string(),
            amount.to_string(),
            format!(
                "{} (executed with {}% slippage)",
                formatted_out,
                slippage_tolerance * 100.0
            ),
            tx_hash,
        ))
    }

    fn parse_amount(&self, amount_str: &str, decimals: u8) -> Result<U256> {
        if let Ok(u) = U256::from_str(amount_str) {
            return Ok(u);
        }
        let f: f64 = amount_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;
        let u = (f * 10f64.powi(decimals as i32)).round();
        Ok(U256::from(u as u128))
    }

    async fn get_quote(
        &self,
        from: Address,
        to: Address,
        amount_in: U256,
        router_addr: Address,
        provider: &impl Provider<Http<Client>>,
    ) -> Result<U256> {
        let params = ISwapRouter::ExactInputSingleParams {
            tokenIn: from,
            tokenOut: to,
            fee: self.config.uniswap_fee_tier as u32,
            recipient: router_addr,
            amountIn: amount_in,
            amountOutMinimum: U256::ZERO,
            sqrtPriceLimitX96: U256::ZERO,
        };

        let router = ISwapRouter::new(router_addr, provider);
        let calldata = router.exactInputSingle(params).calldata().to_owned();

        let tx = TransactionRequest::default()
            .from(Address::ZERO)
            .to(router_addr)
            .input(calldata.into());

        let output = provider.call(&tx).await?;
        Ok(U256::from_be_slice(&output))
    }

    async fn ensure_approval(
        &self,
        token: Address,
        spender: Address,
        amount: U256,
        provider: &impl Provider<Http<Client>>,
    ) -> Result<()> {
        let owner = self.signer.address();
        let contract = IERC20::new(token, provider);
        let current_allowance = contract.allowance(owner, spender).call().await?._0;

        if current_allowance < amount {
            return Err(anyhow::anyhow!(
                "Insufficient allowance. Current: {}, Required: {}. Please pre-approve before execution to ensure single transaction.",
                current_allowance,
                amount
            ));
        }

        Ok(())
    }

    async fn build_swap_tx(
        &self,
        from: Address,
        to: Address,
        router_addr: Address,
        amount_in: U256,
        min_output: U256,
        recipient: Address,
        provider: &impl Provider<Http<Client>>,
    ) -> Result<TransactionRequest> {
        let params = ISwapRouter::ExactInputSingleParams {
            tokenIn: from,
            tokenOut: to,
            fee: self.config.uniswap_fee_tier as u32,
            recipient,
            amountIn: amount_in,
            amountOutMinimum: min_output,
            sqrtPriceLimitX96: U256::ZERO,
        };

        let router = ISwapRouter::new(router_addr, provider);
        let calldata = router.exactInputSingle(params).calldata().to_owned();

        Ok(TransactionRequest::default()
            .from(recipient)
            .to(router_addr)
            .input(calldata.into()))
    }
}

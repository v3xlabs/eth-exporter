use alloy::primitives::{Address, U256};
use alloy::providers::DynProvider;
use alloy::sol;
use tokio::sync::Mutex;

sol! {
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface ERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function name() external view returns (string memory);
        function decimals() external view returns (uint8);
    }
}

pub struct ERC20Token {
    pub address: Address,
    pub name: Mutex<String>,
    pub decimals: Mutex<u8>,
}

impl ERC20Token {
    pub async fn new(address: Address, provider: &DynProvider) -> Self {
        let erc20 = ERC20::new(address, provider);
        let name = erc20.name().call().await.unwrap();
        let decimals = erc20.decimals().call().await.unwrap();
        Self { address, name: Mutex::new(name.to_string()), decimals: Mutex::new(decimals) }
    }

    pub async fn nominal_amount(&self) -> U256 {
        let decimals = *self.decimals.lock().await;
        U256::from(10).pow(U256::from(decimals))
    }

    pub async fn format_amount(&self, amount: U256, precision: usize) -> String {
        let decimals = *self.decimals.lock().await;
        let amount = amount.to_string().parse::<f64>().unwrap();
        let amount = amount / 10_f64.powf(decimals as f64);
        format!("{:.precision$}", amount)
    }
}
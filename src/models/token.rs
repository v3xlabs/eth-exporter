use std::sync::Arc;

use alloy::{primitives::Address, providers::RootProvider, transports::http::Http};
use async_std::sync::Mutex;
use reqwest::Client;

use crate::models::erc20::ERC20;

#[derive(Debug)]
pub struct IndexableTokenERC20 {
    pub address: Address,

    pub erc20: ERC20::ERC20Instance<
        alloy::transports::http::Http<reqwest::Client>,
        Arc<alloy::providers::RootProvider<alloy::transports::http::Http<reqwest::Client>>>,
    >,

    pub name: Mutex<String>,
    pub decimals: Mutex<u8>,
    pub usd_price: Mutex<f64>,
}

impl IndexableTokenERC20 {
    pub async fn new(address: Address, provider: Arc<RootProvider<Http<Client>>>) -> Self {
        let erc20 = ERC20::new(address, provider);

        let me = Self {
            address,
            erc20,
            name: Mutex::new(String::new()),
            decimals: Mutex::new(0),
            usd_price: Mutex::new(0.0),
        };

        futures::join!(me.update_name(), me.update_usd_price(), me.update_decimals());

        me
    }

    pub async fn update_name(&self) {
        let name = self.erc20.name().call().await.expect("Failed to get name");
        let name = name._0;

        *self.name.lock().await = name.to_string();
    }

    pub async fn update_usd_price(&self) {
        let usd_price = 1.0;

        *self.usd_price.lock().await = usd_price;
    }

    pub async fn update_decimals(&self) {
        let decimals = self.erc20.decimals().call().await.expect("Failed to get decimals");
        let decimals = decimals._0;

        *self.decimals.lock().await = decimals;
    }
}

use std::{ops::{Deref, Div}, str::FromStr, sync::Arc};

use alloy::{
    eips::BlockId,
    network::{AnyNetwork, Ethereum, Network},
    primitives::{aliases::U24, Address, Uint, U160, U256},
    providers::{Provider, RootProvider},
    transports::{http::Http, Transport},
};
use async_std::sync::Mutex;
use reqwest::Client;
use uniswap_v3_sdk::{
    prelude::{
        sdk_core::prelude::FractionBase, EphemeralTickMapDataProvider, FeeAmount, Pool,
        FACTORY_ADDRESS,
    },
    utils::ToBig,
};

use crate::{
    models::erc20::{UniswapV2Pool, ERC20}, state::TokenConfigTest, MyProvider
};

use super::erc20::UniswapV3Quoter;

#[derive(Debug)]
pub struct IndexableTokenERC20 {
    pub address: Address,

    pub chain_slug: String,
    pub erc20: ERC20::ERC20Instance<(), Arc<MyProvider>, Ethereum>,
    pub name: Mutex<String>,
    pub decimals: Mutex<u8>,
    pub usd_price: Mutex<f64>,

    pub fixed_rate: Option<f64>,
    pub uniswap_v2: Option<String>,
    pub uniswap_v3: Option<String>,
}

impl IndexableTokenERC20 {
    pub async fn new(config: &TokenConfigTest, chain_slug: String, provider: Arc<MyProvider>) -> Self {
        let erc20 = ERC20::new(config.address.parse().unwrap(), provider);
        let address = config.address.parse().unwrap();

        let me = Self {
            address,
            chain_slug,
            erc20,
            name: Mutex::new(String::new()),
            decimals: Mutex::new(0),
            usd_price: Mutex::new(0 as f64),
            fixed_rate: config.fixed_rate,
            uniswap_v2: config.uniswap_v2.clone(),
            uniswap_v3: config.uniswap_v3.clone(),
        };

        me.update_name().await;
        me.update_decimals().await;
        me.update_usd_price().await;

        me
    }

    pub async fn update_name(&self) {
        let name = self.erc20.name().call().await.expect("Failed to get name");
        let name = name._0;

        *self.name.lock().await = name.to_string();
    }

    pub async fn update_usd_price(&self) {
        if let Some(fixed_rate) = self.fixed_rate {
            *self.usd_price.lock().await = fixed_rate;
            return;
        }

        if let Some(uniswap_v2) = &self.uniswap_v2 {
            let pool = UniswapV2Pool::new(uniswap_v2.parse().unwrap(), self.erc20.provider().clone());
            let reserves = pool.getReserves().call().await.unwrap();
            let reserve0 = reserves.reserve0.to_string().parse::<f64>().unwrap() / 10_u128.pow(6) as f64; // usdc
            let reserve1 = reserves.reserve1.to_string().parse::<f64>().unwrap() / 10_u128.pow(18) as f64; // ens
            let price = reserve0 / reserve1;
            *self.usd_price.lock().await = price;
            return;
        }

        if let Some(uniswap_v3) = &self.uniswap_v3 {
            let usdc_address: Address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                .parse()
                .unwrap();

            let decimals = *self.decimals.lock().await;

            let pool = UniswapV3Quoter::new(
                "0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6"
                    .parse()
                    .unwrap(),
                self.erc20.provider().clone(),
            );
            let weth_address = Address::from_str(uniswap_v3).unwrap();
            let usd_price = pool
                .quoteExactInputSingle(
                    weth_address,
                    usdc_address,
                    FeeAmount::MEDIUM.into(),
                    U256::from(10).pow(U256::from(decimals)),
                    U160::from(0),
                )
                .call()
                .await
                .unwrap();

            println!("usd_price: {:?}", usd_price);

            let amount_in_usdc = usd_price.amountOut;
            let amount_in_usdc = amount_in_usdc.to_string().parse::<f64>().unwrap();
            let amount_in_usd = amount_in_usdc.div(10_f64.powf(6.0));

            *self.usd_price.lock().await = amount_in_usd;
        }
    }

    pub async fn update_decimals(&self) {
        let decimals = self
            .erc20
            .decimals()
            .call()
            .await
            .expect("Failed to get decimals");
        let decimals = decimals._0;

        *self.decimals.lock().await = decimals;
    }
}

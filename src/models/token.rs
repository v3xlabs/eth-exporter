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
    models::erc20::{UniswapV2Pool, ERC20},
    MyProvider,
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
}

impl IndexableTokenERC20 {
    pub async fn new(address: Address, chain_slug: String, provider: Arc<MyProvider>) -> Self {
        let erc20 = ERC20::new(address, provider);

        let me = Self {
            address,
            chain_slug,
            erc20,
            name: Mutex::new(String::new()),
            decimals: Mutex::new(0),
            usd_price: Mutex::new(0 as f64),
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
        if self.chain_slug == "eth"
            && self.address
                == Address::from_str("0xc18360217d8f7ab5e7c516566761ea12ce7f9d72").unwrap()
        {
            println!("Updating USD price for ENS {}", self.address);

            let pool_address: Address = "0xb169c3e8dda6456a18aefa49c58f7f53e120a9b4"
                .parse()
                .unwrap();
            let pool = UniswapV2Pool::new(pool_address, self.erc20.provider().clone());

            let reserves = pool.getReserves().call().await.unwrap();
            let reserve0 =
                reserves.reserve0.to_string().parse::<f64>().unwrap() / 10_u128.pow(6) as f64; // usdc
            let reserve1 =
                reserves.reserve1.to_string().parse::<f64>().unwrap() / 10_u128.pow(18) as f64; // ens

            let price = reserve0 / reserve1;

            let usd_price = price;

            println!("USD price for {} is {}", self.address, usd_price);

            *self.usd_price.lock().await = usd_price;
        }

        let usdcs = [
            Address::from_str("0x98C23E9d8f34FEFb1B7BD6a91B7FF122F4e16F5c").unwrap(),
            Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap(),
        ];
        if self.chain_slug == "eth" && usdcs.contains(&self.address) {
            println!("Updating USD price for USDC {}", self.address);

            let usd_price = 1.0;

            *self.usd_price.lock().await = usd_price;
        }

        if self.chain_slug == "eth"
            && self.address
                == Address::from_str("0xae7ab96520de3a18e5e111b5eaab095312d7fe84").unwrap()
        {
            println!("Updating USD price for stETH {}", self.address);

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
            let weth_address = Address::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")
                .unwrap();
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

        if self.chain_slug == "polygon"
            && self.address
                == Address::from_str("0x625e7708f30ca75bfd92586e17077590c60eb4cd").unwrap()
        {
            println!("Updating USD price for USDC {}", self.address);

            let usd_price = 1.0;

            *self.usd_price.lock().await = usd_price;
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

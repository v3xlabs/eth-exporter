use std::{ops::Deref, str::FromStr, sync::Arc};

use alloy::{
    eips::BlockId,
    network::{AnyNetwork, Ethereum, Network},
    primitives::{aliases::U24, Address, U160, U256},
    providers::{Provider, RootProvider},
    transports::{http::Http, Transport},
};
use async_std::sync::Mutex;
use reqwest::Client;

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

        futures::join!(
            me.update_name(),
            me.update_usd_price(),
            me.update_decimals()
        );

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
            println!("Updating USD price for {}", self.address);

            // quoter 0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6
            // let quoter_address =
            //     Address::from_str("0xb27308f9F90D607463bb33eA1BeBb41C27CE5AB6").unwrap();
            // let usdc_address =
            //     Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap();
            // let amount = U256::from(1000000000000000000 as i64);
            // let fee = U24::from(500);
            // let sqrt_price_limit = U160::from(0);

            // let quoter = UniswapV3Quoter::new(quoter_address, self.erc20.provider().clone());
            // let usd_price = match quoter
            // .quoteExactInputSingle(
            //     self.address,
            //     usdc_address,
            //     fee,
            //     amount,
            //     sqrt_price_limit,
            // )
            // .call()
            // .await {
            //     Ok(usd_price) => usd_price.amountOut,
            //     Err(e) => {
            //         println!("Error getting USD price: {}", e);
            //         return;
            //     }
            // };

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
            println!("Updating USD price for {}", self.address);

            let usd_price = 1.0;

            *self.usd_price.lock().await = usd_price;
        }

        if self.chain_slug == "eth"
            && self.address
                == Address::from_str("0xae7ab96520de3a18e5e111b5eaab095312d7fe84").unwrap()
        {
            println!("Updating USD price for {}", self.address);

            let pool_address: Address = "0x6c83b0feef04139eb5520b1ce0e78069c6e7e2c5"
                .parse()
                .unwrap();
            let USDC_ADDRESS: Address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
                .parse()
                .unwrap();

            let provider = self.erc20.provider().clone();

            // let pool = Pool::<EphemeralTickMapDataProvider>::from_pool_key_with_tick_data_provider(
            //     1,
            //     FACTORY_ADDRESS,
            //     self.address,
            //     USDC_ADDRESS,
            //     FeeAmount::LOW,
            //     provider,
            //     Some(BlockId::latest()),
            // )
            // .await
            // .unwrap;

            // univ3pool 0x6c83b0feef04139eb5520b1ce0e78069c6e7e2c5
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

use std::{env, str::FromStr};

use alloy::primitives::Address;
use prometheus::{
    core::{AtomicU64, GenericGaugeVec},
    Opts, Registry,
};

pub struct AppState {
    pub prometheus: Registry,
    pub balance_of: GenericGaugeVec<AtomicU64>,

    pub wallets: Vec<Address>,
    pub erc20s: Vec<Address>,
}

impl AppState {
    pub fn new() -> Self {
        let prometheus = Registry::new();

        let wallets: Vec<Address> = env::var("WALLET_ADDRESSES")
            .unwrap()
            .split(',')
            .map(|s| Address::from_str(s.trim_start_matches("0x")).unwrap())
            .collect();

        let erc20s: Vec<Address> = env::var("ERC20_ADDRESSES")
            .unwrap()
            .split(',')
            .map(|s| Address::from_str(s.trim_start_matches("0x")).unwrap())
            .collect();

        // let balance_of =
        //     register_gauge_vec!("balance_of", "Balance by user by token", &["user", "token"])
        //         .unwrap();
        let opts = Opts::new("balance_of", "Balance by user by token");
        let balance_of = GenericGaugeVec::new(opts, &["user", "token"]).unwrap();
        prometheus.register(Box::new(balance_of.clone())).unwrap();

        Self {
            prometheus,
            balance_of,
            wallets,
            erc20s,
        }
    }
}

use std::fmt::Display;

use alloy::{primitives::{Address, U256}, providers::DynProvider};
use serde::{Deserialize, Deserializer};

use crate::token::erc20::ERC20Token;

#[derive(Debug, PartialEq, Clone)]
pub enum LocalTokenOrFiat {
    ERC20 { address: Address },
    Fiat { symbol: String },
}

const FIAT_DECIMALS: u32 = 6;

impl LocalTokenOrFiat {
    pub async fn nominal_amount(&self, provider: &DynProvider) -> U256 {
        match self {
            LocalTokenOrFiat::ERC20 { address } => ERC20Token::new(*address, provider).await.nominal_amount().await,
            LocalTokenOrFiat::Fiat {symbol: _} => U256::from(10_u64.pow(FIAT_DECIMALS)),
        }
    }

    pub async fn format_amount(&self, amount: U256, precision: usize, provider: &DynProvider) -> String {
        match self {
            LocalTokenOrFiat::ERC20 { address } => ERC20Token::new(*address, provider).await.format_amount(amount, precision).await,
            // TODO: verify the f64 math vs u256 math with precision offset exponent
            LocalTokenOrFiat::Fiat {symbol: _} => {
                let amount = amount.to_string().parse::<f64>().unwrap() / 10_f64.powf(FIAT_DECIMALS as f64);

                format!("{:.precision$}", amount)
            },
        }
    }

    pub async fn symbol(&self, provider: &DynProvider) -> String {
        match self {
            LocalTokenOrFiat::ERC20 { address } => ERC20Token::new(*address, provider).await.name.lock().await.clone(),
            LocalTokenOrFiat::Fiat {symbol: _} => "fiat".to_string(),
        }
    }
}

impl<'de> Deserialize<'de> for LocalTokenOrFiat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.starts_with("fiat:") {
            let symbol = s.split("fiat:").nth(1).unwrap().to_string();

            Ok(LocalTokenOrFiat::Fiat { symbol })
        } else if s.starts_with("0x") {
            Ok(LocalTokenOrFiat::ERC20 {
                address: s.parse().unwrap(),
            })
        } else {
            Err(serde::de::Error::custom(format!("Invalid token: {}", s)))
        }
    }
}

impl Display for LocalTokenOrFiat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalTokenOrFiat::ERC20 { address } => write!(f, "{}", address),
            LocalTokenOrFiat::Fiat { symbol } => write!(f, "fiat:{}", symbol),
        }
    }
}

impl From<Address> for LocalTokenOrFiat {
    fn from(address: Address) -> Self {
        LocalTokenOrFiat::ERC20 { address }
    }
}

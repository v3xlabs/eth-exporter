use std::fmt::Display;

use alloy::primitives::Address;
use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq, Clone)]
pub enum LocalTokenOrFiat {
    ERC20 { address: Address },
    Fiat { symbol: String },
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

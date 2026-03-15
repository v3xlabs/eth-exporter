use alloy::sol;

sol! {
    #[sol(rpc)]
    contract UniswapV3Factory {
         function slot0() public view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{Address, address};

    use super::*;

    const FACTORY_ADDRESS: Address = address!("0x1F98431c8aD98523631AE4a59f267346ea31F984");
}
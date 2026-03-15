use alloy::{
    primitives::{aliases::U112, Address, U256},
    providers::DynProvider,
    sol,
};

sol! {
   #[sol(rpc)]
   contract UniswapV2Pair {
        function factory() external view returns (address);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function getReserves() public view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function price0CumulativeLast() external view returns (uint);
        function price1CumulativeLast() external view returns (uint);
        function kLast() external view returns (uint);
   }
}

#[derive(Debug)]
pub struct PairInfo {
    pub reserves: (U112, U112, u32),
    pub price0: U256,
    pub price1: U256,
    pub k_last: U256,
    pub token_a: Address,
    pub token_b: Address,
}

pub async fn fetch_pair_info(
    provider: &DynProvider,
    pair_address: Address,
) -> Result<PairInfo, Box<dyn std::error::Error>> {
    let pair = UniswapV2Pair::new(pair_address, provider);
    let fr = &pair;

    let reserves = fr.getReserves().call().await?;
    let price0 = fr.price0CumulativeLast().call().await?;
    let price1 = fr.price1CumulativeLast().call().await?;
    let k_last = fr.kLast().call().await?;
    let token_a = fr.token0().call().await?;
    let token_b = fr.token1().call().await?;

    Ok(PairInfo {
        reserves: (
            reserves.reserve0,
            reserves.reserve1,
            reserves.blockTimestampLast,
        ),
        price0,
        price1,
        k_last,
        token_a,
        token_b,
    })
}

#[cfg(test)]
mod tests {
    use crate::tests::get_test_provider;
    use alloy::primitives::address;

    use super::*;

    #[tokio::test]
    async fn test_fetch_pair_info() {
        let provider = get_test_provider().await;
        let pair_address = address!("0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc");
        let pair_info = fetch_pair_info(provider, pair_address).await.unwrap();

        println!("pair_info: {:?}", pair_info);
    }
}

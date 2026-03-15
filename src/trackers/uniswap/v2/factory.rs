use alloy::primitives::{Address, U256};
use alloy::providers::DynProvider;
use alloy::sol;
use async_stream::stream;
use futures::Stream;

sol! {
   #[sol(rpc)]
   contract UniswapV2Factory {
        function getPair(address tokenA, address tokenB) external view returns (address pair);
        function allPairs(uint) external view returns (address pair);
        function allPairsLength() external view returns (uint);
   }
}

pub fn fetch_all_pairs<'a>(
    provider: &'a DynProvider,
    factory_address: Address,
) -> impl Stream<Item = Address> + 'a {
    stream! {
        let factory = UniswapV2Factory::new(factory_address, provider);
        let fr = &factory;
        let max = fr.allPairsLength().call().await.unwrap();

        let mut state = U256::from(0);
        while state < max {
            
            let pair = fr.allPairs(U256::from(state)).call().await.unwrap();
            
            yield pair;

            state += U256::from(1);
        }
    }
}

pub async fn fetch_pair(
    provider: &DynProvider,
    factory_address: Address,
    token_from: Address,
    token_to: Address,
) -> Result<Address, Box<dyn std::error::Error>> {
    let factory = UniswapV2Factory::new(factory_address, provider);
    let fr = &factory;

    let pair = fr.getPair(token_from, token_to).call().await?;
    Ok(pair)
}

#[cfg(test)]
mod tests {
    use crate::tests::get_test_provider;
    use alloy::primitives::address;
    use futures::StreamExt;

    use super::*;

    const FACTORY_ADDRESS: Address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

    #[tokio::test]
    async fn test_fetch_all_pairs() {
        let provider = get_test_provider().await;
        let pairs = fetch_all_pairs(provider, FACTORY_ADDRESS);

        let first_pairs = pairs.take(5).collect::<Vec<_>>().await;

        let dummy = vec![
            address!("0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc"),
            address!("0x3139ffc91b99aa94da8a2dc13f1fc36f9bdc98ee"),
            address!("0x12ede161c702d1494612d19f05992f43aa6a26fb"),
            address!("0xa478c2975ab1ea89e8196811f51a7b7ade33eb11"),
            address!("0x07f068ca326a469fc1d87d85d448990c8cba7df9"),
        ];

        assert_eq!(first_pairs, dummy)
    }

    #[tokio::test]
    async fn test_fetch_pair() {
        let provider = get_test_provider().await;
        let token_usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let token_weth = address!("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        let pair = fetch_pair(provider, FACTORY_ADDRESS, token_usdc, token_weth)
            .await
            .unwrap();

        assert_eq!(pair, address!("0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc"));
    }
}

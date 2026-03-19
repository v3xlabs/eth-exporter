
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct UniswapV3QuoterConfig {
//     pub pool_address: String,
// }

// impl UniswapV3QuoterConfig {
//     pub async fn quote(
//         &self,
//         quoter: &Quoter,
//         amount_in: &U256,
//         token_out: &TokenIdentity,
//         state: &StateArc,
//         block_number: Option<u64>,
//     ) -> Result<U256, ApiError> {
//         let (network_identity, _) = quoter.token_a.unwrap_evm().ok_or(ApiError::Unsupported(
//             "Token identity is not an EVM token".to_string(),
//         ))?;

//         let pool_address = Address::from_str(&self.pool_address).or(Err(ApiError::BadRequest))?;

//         let provider = state
//             .rpc_manager
//             .get_pool(&network_identity)
//             .get_first_rpc()
//             .ok_or(ApiError::NoEndpointsAvailable)?
//             .get_provider()
//             .ok_or(ApiError::NoEndpointsAvailable)?;

//         let pool = UniswapV3Pool::new(pool_address, provider);

//         let block_number = match block_number {
//             Some(block_number) => BlockNumberOrTag::Number(block_number),
//             None => BlockNumberOrTag::Latest,
//         };

//         let slot0 = pool
//             .slot0()
//             .block(BlockId::Number(block_number))
//             .call()
//             .await?;

//         let sqrt_price_x96 = U512::from(slot0.sqrtPriceX96);

//         // 2^192
//         let q192 = U512::from(1) << 192;
//         // sqrtP^2
//         let sqrt_price_squared = sqrt_price_x96 * sqrt_price_x96;

//         let price0_in_1_raw = (sqrt_price_squared * U512::from(*amount_in)) / q192;
//         let price1_in_0_raw = (q192 * U512::from(*amount_in)) / sqrt_price_squared;

//         let direction = token_out == &quoter.token_b;

//         Ok(U256::from(if direction {
//             price0_in_1_raw
//         } else {
//             price1_in_0_raw
//         }))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

}
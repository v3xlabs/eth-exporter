use alloy::sol;

// erc20 interface
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface ERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function name() external view returns (string memory);
        function decimals() external view returns (uint8);
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface UniswapV3Quoter {
        function quoteExactInputSingle(
            address tokenIn,
            address tokenOut,
            uint24 fee,
            uint256 amountIn,
            uint160 sqrtPriceLimitX96
        ) public override returns (uint256 amountOut);
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface UniswapV2Pool {
        function getReserves() public view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    }
}

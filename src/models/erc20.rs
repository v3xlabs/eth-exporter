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

# eth-exporter

A simple prometheus exporter for ethereum blockchain data.

## Installation

To get started you can easily run eth-exporter using docker-compose.

```yaml
services:
  eth-exporter:
    image: ghcr.io/v3xlabs/eth-exporter:edge
    ports:
      - 3000:3000
    volumes:
      - ./config.toml:/config.toml
```

Et voila, your metrics are now available at [/metrics](http://localhost:3000/metrics).

## Configuration

A basic configuration for your `config.toml` looks like this:

```toml
[tokens.eth]
rpc_url = "https://eth.blockscout.com/api/eth-rpc"
wallets = ["0xCA5F38911a8d3f4F4D4025Dd9483Ba69932a98bD"]

[tokens.eth.usdc]
address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
fixed_rate = 1.0
```

However feel free to steal the [example config.toml](./config.toml) file (update the `wallets` field) and modify it to your liking.

### Specifying a Token Exchange Rate

There are a few ways you can specify what exchange rate you want to use for a token.
Specifying an exchange rate for a token is optional however does provide you with the `balance_of_usd` and `price_in_usd` metrics.

#### Fixed Rate

You can specify a fixed rate for a token by setting the `fixed_rate` field to a value.
This can be helpful for pegged tokens or tokens whose exchange rate you do not want to compute.

```toml
[tokens.eth.usdc]
address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
fixed_rate = 1.0
```

#### Uniswap V2

You can specify a Uniswap V2 pool address to use for the token.
In the below example we are using the [ENS/USDC](https://etherscan.io/address/0xb169c3e8dda6456a18aefa49c58f7f53e120a9b4) pool to calculate ENS/USDC price.

```toml
[tokens.eth.ens]
address = "0xc18360217d8f7ab5e7c516566761ea12ce7f9d72"
uniswap_v2 = "0xb169c3e8dda6456a18aefa49c58f7f53e120a9b4"
```

> For now only `*/USDC` pairs are supported.

#### Uniswap V3

You can specify a Uniswap V3 from token address to use for the token.
In the below example we are using the WETH token address to calculate WETH/USDC price.

> For now only `*/USDC` pairs are supported.

```toml
[tokens.eth.steth]
address = "0xae7ab96520de3a18e5e111b5eaab095312d7fe84"
uniswap_v3 = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
```

## Output

The output is a prometheus metrics endpoint that you can scrape from your prometheus instance.

A sample output using the [example config.toml](./config.toml) file is shown below:

```raw
# HELP balance_of Balance by user by token
# TYPE balance_of gauge
balance_of{chain="eth",token="0x98c23e9d8f34fefb1b7bd6a91b7ff122f4e16f5c",token_name="Aave Ethereum USDC",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 0
balance_of{chain="eth",token="0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",token_name="USD Coin",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 82998.03958
balance_of{chain="eth",token="0xae7ab96520de3a18e5e111b5eaab095312d7fe84",token_name="Liquid staked Ether 2.0",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 0
balance_of{chain="eth",token="0xc18360217d8f7ab5e7c516566761ea12ce7f9d72",token_name="Ethereum Name Service",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 1144.0360760636715
# HELP balance_of_usd Balance by user by token in USD
# TYPE balance_of_usd gauge
balance_of_usd{chain="eth",token="0x98c23e9d8f34fefb1b7bd6a91b7ff122f4e16f5c",token_name="Aave Ethereum USDC",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 0
balance_of_usd{chain="eth",token="0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",token_name="USD Coin",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 82998.03958
balance_of_usd{chain="eth",token="0xae7ab96520de3a18e5e111b5eaab095312d7fe84",token_name="Liquid staked Ether 2.0",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 0
balance_of_usd{chain="eth",token="0xc18360217d8f7ab5e7c516566761ea12ce7f9d72",token_name="Ethereum Name Service",user="0xd8da6bf26964af9d7eed9e03e53415d37aa96045"} 31262.1223869843
# HELP price_of_usd Price in USD
# TYPE price_of_usd gauge
price_of_usd{chain="eth",token="0x98c23e9d8f34fefb1b7bd6a91b7ff122f4e16f5c",token_name="Aave Ethereum USDC"} 1
price_of_usd{chain="eth",token="0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",token_name="USD Coin"} 1
price_of_usd{chain="eth",token="0xae7ab96520de3a18e5e111b5eaab095312d7fe84",token_name="Liquid staked Ether 2.0"} 2739.368131
price_of_usd{chain="eth",token="0xc18360217d8f7ab5e7c516566761ea12ce7f9d72",token_name="Ethereum Name Service"} 27.32616832726908
```

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

Et voila, your metrics are now available at `http://localhost:3000/metrics`.

## Configuration

You can setup your `config.toml` like the following:

```toml
[tokens.eth]
rpc_url = "https://eth.blockscout.com/api/eth-rpc"
wallets = ["0xCA5F38911a8d3f4F4D4025Dd9483Ba69932a98bD"]

[tokens.eth.usdc]
address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
fixed_rate = 1.0
```

### Specifying a Token Exchange Rate

#### Fixed Rate

You can specify a fixed rate for a token by setting the `fixed_rate` field to a value.

```toml
[tokens.eth.usdc]
address = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
fixed_rate = 1.0
```

#### Uniswap V2

You can specify a Uniswap V2 pool address to use for the token.

```toml
[tokens.eth.ens]
address = "0xc18360217d8f7ab5e7c516566761ea12ce7f9d72"
uniswap_v2 = "0xb169c3e8dda6456a18aefa49c58f7f53e120a9b4"
```

> For now only */USDC pairs are supported.

#### Uniswap V3

You can specify a Uniswap V3 from token address to use for the token.
In the below example we are using the WETH token address to calculate WETH/USDC price.

> For now only */USDC pairs are supported.

```toml
[tokens.eth.steth]
address = "0xae7ab96520de3a18e5e111b5eaab095312d7fe84"
uniswap_v3 = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
```

use alloy::providers::DynProvider;
use alloy::providers::Provider;
use alloy::providers::ProviderBuilder;
use tokio::sync::OnceCell;

static PROVIDER: OnceCell<DynProvider> = OnceCell::const_new();

pub async fn get_test_provider() -> &'static DynProvider {
    PROVIDER
        .get_or_init(|| async {
            let provider = ProviderBuilder::new()
                .connect("https://reth-ethereum.ithaca.xyz/rpc")
                .await
                .unwrap();

            provider.erased()
        })
        .await
}

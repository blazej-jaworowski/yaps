use yaps_codecs::JsonCodec;
use yaps_core::{FuncProvider as _, Result, codec::Codec as _, local_hub::LocalHub};
use yaps_macros::yaps_plugin;

#[yaps_plugin]
mod adder {
    #[derive(Default)]
    pub struct Adder;

    #[yaps_export(namespace = "auto")]
    impl Adder {
        fn add(&self, a: i32, b: i32) -> i32 {
            a + b
        }

        #[yaps_export(namespace = "Subber", id = "sub")]
        fn sub_test(&self, a: i32, b: i32) -> i32 {
            a - b
        }
    }
}

#[yaps_plugin]
mod multiplier {
    use yaps_core::Result;

    #[derive(Default)]
    pub struct Multiplier;

    #[yaps_extern(namespace = "Adder")]
    impl Multiplier {
        async fn add(&self, a: i32, b: i32) -> i32;

        #[yaps_extern(namespace = "Subber")]
        async fn sub(&self, a: i32, b: i32) -> i32;
    }

    impl Multiplier {
        #[yaps_export(id = "mult")]
        async fn mult(&self, a: i32, b: i32) -> Result<i32> {
            let mut sum = 0;
            for _ in 0..b {
                sum = self.add(sum, a).await?;
            }
            Ok(sum)
        }

        #[yaps_export(id = "div")]
        async fn div(&self, mut a: i32, b: i32) -> Result<i32> {
            let mut i = 0;
            while a >= b {
                i = self.add(i, 1).await?;
                a = self.sub(a, b).await?;
            }
            Ok(i)
        }
    }
}

#[tokio::test]
async fn single_provider_test() -> Result<()> {
    let mut hub = LocalHub::new();

    let adder = adder::AdderWrapper::new(adder::Adder::default(), JsonCodec);
    let multiplier =
        multiplier::MultiplierWrapper::new(multiplier::Multiplier::default(), JsonCodec);

    hub.add_provider(adder).await?;
    hub.add_plugin(multiplier).await?;

    let func = hub.get_func("mult").await?;

    let codec = JsonCodec;
    let data = codec.encode((12, 3))?;
    let result = func.call(data).await?;
    let result: Result<i32> = codec.decode(result)?;

    assert_eq!(result, Ok(36));

    let func = hub.get_func("div").await?;

    let codec = JsonCodec;
    let data = codec.encode((13, 3))?;
    let result = func.call(data).await?;
    let result: Result<i32> = codec.decode(result)?;

    assert_eq!(result, Ok(4));

    Ok(())
}

use yaps_core::{
    Result,
    local_orchestrator::LocalOrchestrator,
    serializer_deserializer::SerializerDeserializer,
    FuncProvider,
};
use yaps_macros::yaps_plugin;
use yaps_serdes::JsonSerde;

struct Adder;

#[yaps_plugin]
impl Adder {
    
    #[yaps_export(id = "Adder::add")]
    fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }

    #[yaps_export(id = "Adder::sub")]
    fn sub(&self, a: i32, b: i32) -> i32 {
        a - b
    }

}

struct Multiplier;

#[yaps_plugin]
impl Multiplier {
    
    #[yaps_extern(id = "Adder::add")]
    async fn add(a: i32, b: i32) -> i32;

    #[yaps_extern(id = "Adder::sub")]
    async fn sub(a: i32, b: i32) -> i32;

    #[yaps_export(id = "Multiplier::mult")]
    async fn mult(&self, ext: YapsExtern, a: i32, b: i32) -> Result<i32> {
        let mut sum = 0;
        for _ in 0..b {
            sum = ext.add(sum, a).await?;
        }
        Ok(sum)
    }

    #[yaps_export(id = "Multiplier::div")]
    async fn div(&self, ext: YapsExtern, mut a: i32, b: i32) -> Result<i32> {
        let mut i = 0;
        while a > 0 {
            i = ext.add(i, 1).await?;
            a = ext.sub(a, b).await?;
        }
        Ok(i)
    }

}

#[tokio::test]
async fn single_provider_test() -> Result<()> {
    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::new();

    let adder = AdderWrapper::wrap(Adder, JsonSerde);
    let multiplier = MultiplierWrapper::wrap(Multiplier, JsonSerde);

    orchestrator.add_provider(adder).await?;
    orchestrator.add_plugin(multiplier).await?;

    let func = orchestrator.get_func(&"Multiplier::mult".to_string()).await?;

    let serde = JsonSerde;
    let data = serde.serialize((12, 3))?;
    let result = func.call(data).await?;
    let result: Result<i32> = serde.deserialize(result)?;

    assert_eq!(result, Ok(36));

    Ok(())
}

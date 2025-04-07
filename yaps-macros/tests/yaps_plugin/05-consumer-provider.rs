use yaps_macros::yaps_plugin;
use yaps_serdes::JsonSerde;

use yaps_core::{
    Result,
    FuncConsumer, FuncProvider,
    YapsData,
};


fn check_provider<Data: YapsData>(_: &impl FuncProvider<Data>) {}
fn check_consumer<Data: YapsData>(_: &impl FuncConsumer<Data>) {}


struct TestStruct;

#[yaps_plugin]
impl TestStruct {

    /* TODO: This node gets parsed as ImplItemFn (don't know why)
    #[yaps_extern(id = "extern_func1")]
    async fn extern_func_empty();
    */

    #[yaps_extern(id = "extern_func2")]
    async fn extern_func_single_arg(a: String);

    #[yaps_extern(id = "extern_func3")]
    async fn extern_func(a: i32, b: &str) -> String;


    #[yaps_export(id = "provide_test_func")]
    fn consume_func(&self, a: i32) -> i32 { a }

    #[yaps_export(id = "provide_test_func2")]
    fn consume_func2(&self, _s: String, _s2: String) -> i32 { 0 }

    async fn consume_func_use_extern(&self, ext: YapsExtern) -> Result<()> {
        // ext.extern_func_empty();
        ext.extern_func_single_arg("test".to_string()).await?;
        let _s: String = ext.extern_func(1, "test").await?;
        Ok(())
    }

    #[yaps_export(id = "consume_provide_test_func")]
    async fn consume_export_func(&self, ext: YapsExtern, _a: i32) -> Result<i32> {
        // ext.extern_func_empty();
        ext.extern_func_single_arg("test".to_string()).await?;
        let _s: String = ext.extern_func(1, "test").await?;
        Ok(123)
    }

}


#[tokio::main]
async fn main() {

    let test_struct = TestStruct;
    let wrapper = TestStructWrapper::wrap(test_struct, JsonSerde);

    check_provider(&wrapper);
    check_consumer(&wrapper);

    let funcs = wrapper.provided_funcs().await
        .expect("Unexpected provided_funcs error");

    assert_eq!(funcs, vec![
        "provide_test_func",
        "provide_test_func2",
        "consume_provide_test_func",
    ]);

}

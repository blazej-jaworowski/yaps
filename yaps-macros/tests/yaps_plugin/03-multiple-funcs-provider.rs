use yaps_macros::yaps_plugin;
use yaps_serdes::JsonSerde;

use yaps_core::{
    FuncConsumer, FuncProvider,
    YapsData,
};


fn check_provider<Data: YapsData>(_: &impl FuncProvider<Data>) {}
fn check_consumer<Data: YapsData>(_: &impl FuncConsumer<Data>) {}


struct TestStruct;

#[yaps_plugin]
impl TestStruct {

    #[yaps_export(id = "provide_test_func")]
    fn consume_func(&self, a: i32) -> i32 { a }

    #[yaps_export(id = "provide_test_func2")]
    fn consume_func2(&self, _s: String, _s2: String) -> i32 { 0 }

}


#[tokio::main]
async fn main() {

    let test_struct = TestStruct;
    let wrapper = TestStructWrapper::wrap(test_struct, JsonSerde);

    check_provider(&wrapper);
    check_consumer(&wrapper);

    let funcs = wrapper.provided_funcs().await
        .expect("Unexpected provided_funcs error");

    assert_eq!(funcs, vec!["provide_test_func", "provide_test_func2"]);

}

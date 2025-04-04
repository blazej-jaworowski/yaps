use yaps_macros::yaps_plugin;
use yaps_serdes::JsonSerde;

use yaps_core::{
    FuncConsumer, FuncProvider,
};


fn check_provider<Data>(_: &impl FuncProvider<Data>) {}
fn check_consumer<Data>(_: &impl FuncConsumer<Data>) {}


struct TestStruct;

#[yaps_plugin]
impl TestStruct {

    #[yaps_export(id = "provide_test_func")]
    fn provide_func(&self, _: String, _: i32) {}

}


fn main() {

    let test_struct = TestStruct;
    let wrapper = TestStructWrapper::wrap(test_struct, JsonSerde);

    check_provider(&wrapper);
    check_consumer(&wrapper);

    let funcs = wrapper.provided_funcs()
        .expect("Unexpected provided_funcs error");

    assert_eq!(funcs, vec!["provide_test_func"]);

}

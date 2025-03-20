use yaps_core::{
    Result, Error,
    plugin_connector::WithSerde,
    orchestrator::{LocalOrchestrator, Orchestrator},

    serde::{Serialize, de::DeserializeOwned},
};

use yaps_macros::{plugin_connector, plugin_func};

use std::io::Cursor;

struct TestPlugin;

impl WithSerde<Vec<u8>> for TestPlugin {

    fn serialize<S: Serialize>(&self, obj: S) -> Result<Vec<u8>> {
        let s = serde_json::to_string(&obj).map_err(|e| {
            Error::SerializeError(e.to_string())
        })?;
        Ok(s.into())
    }

    fn deserialize<D: DeserializeOwned>(&self, data: Vec<u8>) -> Result<D> {
        let c = Cursor::new(data);
        serde_json::from_reader(c).map_err(|e: serde_json::Error| {
            Error::DeserializeError(e.to_string())
        })
    }

}

#[plugin_connector(Vec<u8>)]
impl TestPlugin {

    #[plugin_func]
    fn echo(&self, s: String) -> String {
        s
    }

    #[plugin_func]
    fn reverse(&self, s: String) -> String {
        s.chars().rev().collect()
    }

    #[plugin_func]
    fn add(&self, a: i64, b: i64) -> i64 {
        a + b
    }

    #[plugin_func]
    fn format(&self, s: String, b: i64) -> String {
        format!("{s} and {b}")
    }

    #[plugin_func]
    fn error(&self) -> std::result::Result<(), String> {
        Err("Error :(".to_string())
    }

}

#[test]
fn string_call_test() -> Result<()> {
    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::default();
    let plugin = TestPlugin;

    orchestrator.register_plugin(plugin)?;

    let reverse_func = orchestrator.get_func(&"reverse".to_string())?;
    let echo_func = orchestrator.get_func(&"echo".to_string())?;

    let in_data: Vec<u8> = "\"dupa\"".into();
    let out_data: Vec<u8> = in_data.clone();
    let reversed_data: Vec<u8> = in_data.clone().into_iter().rev().collect();

    assert_eq!((*reverse_func)(in_data.clone())?, reversed_data);
    assert_eq!((*echo_func)(in_data.clone())?, out_data);

    Ok(())
}

#[test]
fn int_call_test() -> Result<()> {
    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::default();
    let plugin = TestPlugin;

    orchestrator.register_plugin(plugin)?;

    let add_func = orchestrator.get_func(&"add".to_string())?;

    let in_data: Vec<u8> = "[1, 2]".into();
    let out_data: Vec<u8> = "3".into();

    assert_eq!((*add_func)(in_data)?, out_data);

    Ok(())
}

#[test]
fn mixed_call_test() -> Result<()> {
    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::default();
    let plugin = TestPlugin;

    orchestrator.register_plugin(plugin)?;

    let format_func = orchestrator.get_func(&"format".to_string())?;

    let in_data: Vec<u8> = "[\":)\", 123]".into();
    let out_data: Vec<u8> = "\":) and 123\"".into();

    assert_eq!((*format_func)(in_data)?, out_data);

    Ok(())
}

#[test]
fn error_test() -> Result<()> {
    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::default();
    let plugin = TestPlugin;

    orchestrator.register_plugin(plugin)?;

    let error_func = orchestrator.get_func(&"error".to_string())?;

    let in_data: Vec<u8> = "null".into();
    let out_data: Vec<u8> = "{\"Err\":\"Error :(\"}".into();

    assert_eq!((*error_func)(in_data)?, out_data);

    Ok(())
}

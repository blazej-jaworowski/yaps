use yaps_core::{
    Result, Error,
    plugin_connector::{WithSerde, SerializerDeserializer},
    FunctionHandle,
    orchestrator::{LocalOrchestrator, Orchestrator},

    serde::{Serialize, de::DeserializeOwned},
};

use yaps_macros::*;

use std::{io::Cursor, cell::RefCell};

struct JsonSerde;

impl SerializerDeserializer<Vec<u8>> for JsonSerde {

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

#[with_serde(data = Vec<u8>, serde_type = JsonSerde)]
struct Adder<'a> {
    calculate_handle: RefCell<FunctionHandle<'a, Vec<u8>>>,
}

#[yaps_plugin(data = Vec<u8>)]
impl<'plugin> Adder<'plugin> {

    fn new() -> Adder<'plugin> {
        Adder {
            calculate_handle: RefCell::new(Box::new(|_| {
                Err(Error::FunctionNotInitialized("subtract".into()))
            })),
        }
    }

    #[yaps_init]
    fn init(&self, orchestrator: &dyn Orchestrator<'plugin, Vec<u8>>) -> Result<()> {
        *self.calculate_handle.borrow_mut() = orchestrator.get_func(("Calculator".into(), "calculate".into()))?;
        Ok(())
    }

    #[yaps_func]
    fn add(&self, a: i64, b: i64) -> i64 {
        a + b
    }

    #[yaps_func]
    fn subtract(&self, a: i64, b: i64) -> i64 {
        a - b
    }

    #[yaps_func]
    fn calculate(&self, a: i64, b: i64) -> Result<i64> {
        let a = a + 1;
        let b = b + 1;

        let serde = self.serde();
        let args = serde.serialize((a, b))?;

        let result = (*self.calculate_handle.borrow())(args)?;

        self.serde().deserialize(result)?
    }

}

#[with_serde(data = Vec<u8>, serde_type = JsonSerde)]
struct Calculator<'a> {
    add_handle: RefCell<FunctionHandle<'a, Vec<u8>>>,
    subtract_handle: RefCell<FunctionHandle<'a, Vec<u8>>>,
}

#[yaps_plugin(data = Vec<u8>)]
impl<'plugin> Calculator<'plugin> {

    #[yaps_extern_func]
    fn subtract(a: i64, b: i64) -> i64;

    #[yaps_extern_func]
    fn add(a: i64, b: i64) -> i64;

    fn new() -> Calculator<'plugin> {
        Calculator {
            add_handle: RefCell::new(Box::new(|_| {
                Err(Error::FunctionNotInitialized("add".into()))
            })),
            subtract_handle: RefCell::new(Box::new(|_| {
                Err(Error::FunctionNotInitialized("subtract".into()))
            })),
        }
    }

    #[yaps_init]
    fn init(&self, orchestrator: &dyn Orchestrator<'plugin, Vec<u8>>) -> Result<()> {
        Ok(())
    }

    #[yaps_func]
    fn calculate(&self, a: i64, b: i64) -> Result<i64> {

        let a = self.add(a, a)?;
        let a = self.add(a, a)?;

        let b = self.add(b, b)?;

        self.subtract(a, b)

    }

}

fn main() -> Result<()> {
    let serde = JsonSerde;

    let mut orchestrator = LocalOrchestrator::<Vec<u8>>::default();

    let adder = Adder::new().wrap();
    let calculator = Calculator::new().wrap();

    orchestrator.register_plugin("Calculator".into(), Box::new(calculator))?;
    orchestrator.register_plugin("Adder".into(), Box::new(adder))?;

    orchestrator.init()?;

    let add_func = orchestrator.get_func(("Adder".into(), "calculate".into()))?;

    let in_data = (1, 2);

    let result = (*add_func)(serde.serialize(in_data)?)?;
    let result: Result<i64> = serde.deserialize(result)?;

    assert_eq!(result, Ok(2));

    Ok(())
}

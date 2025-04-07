use crate::Result;

use std::{
    pin::Pin,
    sync::Arc,
    future::Future,
};
use async_trait::async_trait;


pub trait YapsData: Send + 'static {}

impl YapsData for Vec<u8> {}
impl YapsData for String {}


pub type FunctionId = String;

type AsyncReturn<D> = Pin<Box<dyn Future<Output = Result<D>> + Send>>;

#[derive(Clone)]
pub struct FunctionHandle<D: YapsData> {
    func: Arc<dyn Fn(D) -> AsyncReturn<D> + Send + Sync>,
}

impl<D: YapsData> FunctionHandle<D> {

    pub async fn call(&self, args: D) -> Result<D> {
        (*self.func)(args).await
    }

    pub fn new<F: Fn(D) -> AsyncReturn<D> + Send + Sync + 'static>(func: F) -> Self {
        Self { func: Arc::new(func) }
    }

}

#[async_trait]
pub trait FuncProvider<D: YapsData>: Send + Sync {

    async fn provided_funcs(&self) -> Result<Vec<FunctionId>>;
    async fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<D>>;

}

#[async_trait]
pub trait FuncConsumer<D: YapsData>: Send + Sync {

    async fn connect(&mut self, provider: &dyn FuncProvider<D>) -> Result<()>;

}

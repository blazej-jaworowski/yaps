use crate::{FuncHandle, FuncMetadata, FuncProvider, Result, YapsData, func_handle::SimpleHandle};

use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
pub struct SingleProvider<D: YapsData, F: Fn(D) -> Result<D> + Send + Sync + 'static> {
    _marker: std::marker::PhantomData<fn(D)>,
    id: String,
    func: Arc<F>,
}

impl<D: YapsData, F: Fn(D) -> Result<D> + Send + Sync + 'static> SingleProvider<D, F> {
    pub fn new(id: String, func: F) -> Self {
        SingleProvider {
            _marker: std::marker::PhantomData,
            id,
            func: Arc::new(func),
        }
    }
}

#[async_trait]
impl<D: YapsData, F: Fn(D) -> Result<D> + Send + Sync + 'static> FuncProvider<D>
    for SingleProvider<D, F>
{
    async fn provided_funcs(&self) -> Result<Vec<FuncMetadata>> {
        Ok(vec![FuncMetadata {
            id: self.id.clone(),
        }])
    }

    async fn get_func(&self, func: &str) -> Result<Box<dyn FuncHandle<D>>> {
        if func == self.id {
            let func_clone = self.func.clone();
            Ok(Box::new(SimpleHandle::new(move |args| func_clone(args))))
        } else {
            Err(crate::Error::FunctionNotFound(func.to_string()))
        }
    }
}

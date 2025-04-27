use crate::{FuncHandle, Result};

use async_trait::async_trait;

pub trait YapsData: Send + 'static {}

impl YapsData for Vec<u8> {}
impl YapsData for String {}

#[derive(Debug, Clone)]
pub struct FuncMetadata {
    pub id: String,
}

#[async_trait]
pub trait FuncProvider<D: YapsData>: Send + Sync {
    async fn provided_funcs(&self) -> Result<Vec<FuncMetadata>>;
    async fn get_func(&self, id: &str) -> Result<Box<dyn FuncHandle<D>>>;
}

#[async_trait]
pub trait FuncConsumer<D: YapsData>: Send + Sync {
    async fn connect(&self, provider: &dyn FuncProvider<D>) -> Result<()>;
}

#[async_trait]
impl<D, T, U> FuncProvider<D> for U
where
    D: YapsData,
    T: FuncProvider<D>,
    U: std::ops::Deref<Target = T> + Send + Sync,
{
    async fn provided_funcs(&self) -> Result<Vec<FuncMetadata>> {
        self.deref().provided_funcs().await
    }

    async fn get_func(&self, id: &str) -> Result<Box<dyn FuncHandle<D>>> {
        self.deref().get_func(id).await
    }
}

#[async_trait]
impl<D, T, U> FuncConsumer<D> for U
where
    D: YapsData,
    T: FuncConsumer<D>,
    U: std::ops::Deref<Target = T> + Send + Sync,
{
    async fn connect(&self, provider: &dyn FuncProvider<D>) -> Result<()> {
        self.deref().connect(provider).await
    }
}

use crate::{
    codec::{Codec, DecodeFor, EncodeFor},
    Result, YapsData,
};

use async_trait::async_trait;

#[async_trait]
pub trait FuncHandle<D: YapsData>: Send + Sync {
    async fn call(&self, args: D) -> Result<D>;

    async fn call_with_codec<C, A, R>(&self, codec: &C, args: A) -> Result<R>
    where
        Self: Sized,
        A: Send,
        C: Codec<Data = D> + EncodeFor<C, A> + DecodeFor<C, R>,
    {
        let data_in = codec.encode(args)?;
        let data_out = self.call(data_in).await?;
        codec.decode(data_out)
    }
}

#[async_trait]
impl<D, T, U> FuncHandle<D> for U
where
    D: YapsData,
    T: FuncHandle<D> + ?Sized,
    U: std::ops::Deref<Target = T> + Send + Sync,
{
    async fn call(&self, args: D) -> Result<D> {
        self.deref().call(args).await
    }
}

pub struct SimpleHandle<D: YapsData, F: FnMut(D) -> Result<D> + Send + Sync> {
    _marker: std::marker::PhantomData<fn(D)>,
    func: F,
}

impl<D: YapsData, F: Fn(D) -> Result<D> + Send + Sync> SimpleHandle<D, F> {
    pub fn new(func: F) -> Self {
        SimpleHandle {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<D: YapsData, F: Fn(D) -> Result<D> + Send + Sync> FuncHandle<D> for SimpleHandle<D, F> {
    async fn call(&self, args: D) -> Result<D> {
        (self.func)(args)
    }
}

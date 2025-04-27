use crate::{Result, YapsData};
use async_trait::async_trait;

#[async_trait]
pub trait Codec: Send + Sync {
    type Data: YapsData;

    fn encode<E>(&self, obj: E) -> Result<Self::Data>
    where
        Self: EncodeFor<Self, E>,
    {
        <Self as EncodeFor<Self, E>>::encode(self, obj)
    }

    fn decode<D>(&self, data: Self::Data) -> Result<D>
    where
        Self: DecodeFor<Self, D>,
    {
        <Self as DecodeFor<Self, D>>::decode(self, data)
    }
}

pub trait EncodeFor<C: Codec + ?Sized, E> {
    fn encode(codec: &C, obj: E) -> Result<C::Data>;
}

pub trait DecodeFor<C: Codec + ?Sized, D> {
    fn decode(codec: &C, data: C::Data) -> Result<D>;
}

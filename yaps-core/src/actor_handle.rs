use crate::{codec::{Codec, DecodeFor, EncodeFor}, Error, FuncHandle, Result, YapsData};

use std::{future::Future, pin::Pin, sync::Arc};
use async_trait::async_trait;
use tokio::{
    task::JoinHandle,
    sync::{mpsc, oneshot},
};

struct ActorCall<D> {
    args: D,
    tx_ret: oneshot::Sender<Result<D>>,
}

pub struct ActorHandle<D> {
    tx_call: mpsc::UnboundedSender<ActorCall<D>>,
}

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

impl<D: YapsData> ActorHandle<D> {
    pub fn spawn<F>(func: F) -> Result<(Self, JoinHandle<Result<()>>)>
    where
        F: Fn(D) -> AsyncResult<D> + Send + Sync + 'static,
    {
        let (tx_call, mut rx_call) = mpsc::unbounded_channel::<ActorCall<D>>();

        let join_handle = tokio::spawn(async move {
            while let Some(call) = rx_call.recv().await {
                let result = func(call.args).await;

                if  call.tx_ret.send(result).is_err() {
                    // TODO: Log return send failure
                }
            }
            Ok(())
        });

        Ok((
            Self { tx_call },
            join_handle,
        ))
    }

    pub fn spawn_with_codec<C, F, A, R>(func: F, codec: Arc<C>) -> Result<(Self, JoinHandle<Result<()>>)>
    where
        C: Codec<Data = D> + DecodeFor<C, A> + EncodeFor<C, R> + 'static,
        F: Fn(A) -> AsyncResult<R> + Send + Sync + 'static,
    {
        let func = Arc::new(func);

        let codec_func = move |args| -> AsyncResult<D> {
            let codec = codec.clone();
            let func = func.clone();

            Box::pin(async move {
                let args = codec.decode(args)?;
                let result = func(args).await?;
                codec.encode(result)
            })
        };

        Self::spawn(codec_func)
    }
}

#[async_trait]
impl<D: YapsData> FuncHandle<D> for ActorHandle<D> {
    async fn call(&self, args: D) -> Result<D> {
        let (tx, rx) = oneshot::channel();
        let call = ActorCall {
            args,
            tx_ret: tx,

        };
        self.tx_call
            .send(call)
            .map_err(|e| Error::ChannelSend(e.to_string()))?;

        rx.await.map_err(|_| Error::HandlerInvalidated)?
    }
}

use crate::{Result, Error};
use crate::{
    FunctionId, FunctionHandle, 
    YapsData,
    FuncProvider, FuncConsumer,
};

use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::Mutex;
use async_trait::async_trait;


pub struct LocalOrchestrator<D: YapsData> {

    providers: Vec<Arc<Mutex<dyn FuncProvider<D>>>>,
    consumers: Vec<Arc<Mutex<dyn FuncConsumer<D>>>>,

}

impl<D: YapsData> Default for LocalOrchestrator<D> {
    fn default() -> Self {
        Self { providers: Vec::new(), consumers: Vec::new() }
    }
}

#[async_trait]
impl<D: YapsData> FuncProvider<D> for LocalOrchestrator<D> {
    
    async fn provided_funcs(&self) -> Result<Vec<FunctionId>> {
        let provided_funcs = self.providers.iter()
            .map(|p| async {
                p.lock().await.provided_funcs().await
            });

        let provided_funcs = join_all(provided_funcs).await;

        let provided_funcs: Vec<_> = provided_funcs.into_iter()
            .filter_map(|res| res.ok())
            .flatten()
            .collect();

        Ok(provided_funcs)
    }

    async fn get_func(&self, id: &FunctionId) -> Result<FunctionHandle<D>> {
        let providers = self.providers.iter()
            .map(|p| async {
                let funcs = p.lock().await.provided_funcs().await;
                (p.clone(), funcs)
            });

        let providers = join_all(providers).await;

        let providers: Vec<_> = providers.into_iter()
            .filter_map(|(p, res)| {
                if res.is_ok_and(|r| r.contains(id)) {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();


        if providers.is_empty() {
            return Err(Error::FunctionNotFound(id.into()));
        }

        // TODO: Maybe handle a case where there are multiple plugins

        let provider = providers[0].lock().await;
        provider.get_func(id).await
    }

}

#[async_trait]
impl<D: YapsData> FuncConsumer<D> for LocalOrchestrator<D> {

    async fn connect(&mut self, provider: &dyn FuncProvider<D>) -> Result<()> {
        let results = self.consumers.iter_mut()
            .map(|c| async { c.lock().await.connect(provider).await });

        join_all(results).await;

        Ok(())
    }

}

impl<D: YapsData> LocalOrchestrator<D> {

    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add_provider(&mut self, provider: impl FuncProvider<D> + 'static) -> Result<()> {
        self.connect(&provider).await?;

        self.providers.push(Arc::new(Mutex::new(provider)));
        Ok(())
    }

    pub async fn add_consumer(&mut self, mut consumer: impl FuncConsumer<D> + 'static) -> Result<()> {
        consumer.connect(self).await?;

        self.consumers.push(Arc::new(Mutex::new(consumer)));
        Ok(())
    }

    pub async fn add_plugin(&mut self, mut cp: impl FuncProvider<D> + FuncConsumer<D> + 'static) -> Result<()> {
        self.connect(&cp).await?;
        cp.connect(self).await?;

        let cp = Arc::new(Mutex::new(cp));

        self.providers.push(cp.clone());
        self.consumers.push(cp);
        Ok(())
    }

}

use crate::{Error, Result};
use crate::{FuncConsumer, FuncHandle, FuncMetadata, FuncProvider, YapsData};

use std::sync::Arc;

use async_trait::async_trait;

struct Provider<D> {
    pub provider: Arc<dyn FuncProvider<D>>,
    pub funcs: Vec<FuncMetadata>,
}

type Consumer<D> = Arc<dyn FuncConsumer<D>>;

// TODO: Implement Debug
#[allow(missing_debug_implementations)]
pub struct LocalHub<D: YapsData> {
    providers: Vec<Provider<D>>,
    consumers: Vec<Consumer<D>>,
}

impl<D: YapsData> Default for LocalHub<D> {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            consumers: Vec::new(),
        }
    }
}

// TODO: Implementation of LocalHub should be parallelized

#[async_trait]
impl<D: YapsData> FuncProvider<D> for LocalHub<D> {
    async fn provided_funcs(&self) -> Result<Vec<FuncMetadata>> {
        let funcs: Vec<_> = self
            .providers
            .iter()
            .flat_map(|p| &p.funcs)
            .cloned()
            .collect();

        Ok(funcs)
    }

    async fn get_func(&self, id: &str) -> Result<Box<dyn FuncHandle<D>>> {
        let mut providers = self
            .providers
            .iter()
            .filter(|p| p.funcs.iter().any(|f| f.id == id));

        let provider = providers
            .next()
            .ok_or(Error::FunctionNotFound(id.to_string()))?;

        let func = provider.provider.get_func(id).await?;

        Ok(func)
    }
}

#[async_trait]
impl<D: YapsData> FuncConsumer<D> for LocalHub<D> {
    async fn connect(&self, provider: &dyn FuncProvider<D>) -> Result<()> {
        for consumer in self.consumers.iter() {
            consumer.connect(provider).await?;
        }

        Ok(())
    }
}

impl<D: YapsData> LocalHub<D> {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add_provider(&mut self, provider: impl FuncProvider<D> + 'static) -> Result<()> {
        self.connect(&provider).await?;

        let funcs = provider.provided_funcs().await?;

        self.providers.push(Provider {
            provider: Arc::new(provider),
            funcs,
        });
        Ok(())
    }

    pub async fn add_consumer(&mut self, consumer: impl FuncConsumer<D> + 'static) -> Result<()> {
        consumer.connect(self).await?;

        self.consumers.push(Arc::new(consumer));
        Ok(())
    }

    pub async fn add_plugin(
        &mut self,
        cp: impl FuncProvider<D> + FuncConsumer<D> + 'static,
    ) -> Result<()> {
        self.connect(&cp).await?;
        cp.connect(self).await?;

        let cp = Arc::new(cp);

        let funcs = cp.provided_funcs().await?;
        let provider = Provider {
            provider: cp.clone(),
            funcs,
        };

        self.providers.push(provider);
        self.consumers.push(cp);
        Ok(())
    }
}

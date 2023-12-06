use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use spin_trigger::{cli::NoArgs, EitherInstance, TriggerAppEngine, TriggerExecutor};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

wasmtime::component::bindgen!({
    world: "spin-cron",
    path: "cron.wit",
    async: true
});

use fermyon::spin_cron::cron_types as cron;

pub(crate) type RuntimeData = ();

pub struct CronTrigger {
    engine: TriggerAppEngine<Self>,
    cron_components: Vec<Component>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CronTriggerConfig {
    pub component: String,
    pub cron_expression: String,
}

#[derive(Clone, Debug)]
struct Component {
    pub id: String,
    pub cron_expression: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
}

#[async_trait]
impl TriggerExecutor for CronTrigger {
    const TRIGGER_TYPE: &'static str = "cron";
    type RuntimeData = RuntimeData;
    type TriggerConfig = CronTriggerConfig;
    type RunConfig = NoArgs;

    async fn new(engine: TriggerAppEngine<Self>) -> Result<Self> {
        let cron_components = engine
            .trigger_configs()
            .map(|(_, config)| Component {
                id: config.component.clone(),
                cron_expression: config.cron_expression.clone(),
            })
            .collect();
        Ok(Self {
            engine,
            cron_components,
        })
    }

    async fn run(self, _config: Self::RunConfig) -> Result<()> {
        let components = self.cron_components;
        let engine = Arc::new(self.engine);
        Self::init_cron_scheduler(engine, components).await
    }
}

impl CronTrigger {
    async fn init_cron_scheduler(
        engine: Arc<TriggerAppEngine<Self>>,
        components: Vec<Component>,
    ) -> anyhow::Result<()> {
        let mut sched = JobScheduler::new().await?;
        for component in components {
            let id = component.id.clone();
            tracing::info!("Adding component  \"{id}\" to job scheduler");
            let engine = engine.clone();
            sched
                .add(Job::new_async(
                    component.cron_expression.clone().as_str(),
                    move |_, _| {
                        let processor = CronEventProcessor::new(engine.clone(), component.clone());
                        let timestamp: u64 = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        Box::pin(async move {
                            _ = processor
                                .handle_cron_event(cron::Metadata { timestamp })
                                .await;
                        })
                    },
                )?)
                .await?;
        }

        sched.start().await?;
        tracing::info!("Job scheduler started");

        // Handle Ctrl + c
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            tracing::info!("Ctrl+C received - Terminating");
            let _ = tx.send(());
        });
        rx.await?;

        sched.shutdown().await?;
        tracing::info!("Job scheduler stopped");

        Ok(())
    }
}

pub struct CronEventProcessor {
    engine: Arc<TriggerAppEngine<CronTrigger>>,
    component: Component,
}

impl CronEventProcessor {
    fn new(engine: Arc<TriggerAppEngine<CronTrigger>>, component: Component) -> Self {
        Self { engine, component }
    }

    async fn handle_cron_event(&self, metadata: cron::Metadata) -> anyhow::Result<()> {
        // Load the guest...
        let (instance, mut store) = self.engine.prepare_instance(&self.component.id).await?;
        let EitherInstance::Component(instance) = instance else {
            unreachable!()
        };
        let instance = SpinCron::new(&mut store, &instance)?;
        // ...and call the entry point
        let res = instance
            .call_handle_cron_event(&mut store, metadata)
            .await
            .context("cron handler trapped")?;
        res.map_err(|e| {
            tracing::error!("Component {} failed: {e}", self.component.id);
            anyhow!("Component {} failed: {e}", self.component.id)
        })
    }
}

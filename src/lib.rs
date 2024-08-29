use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use spin_trigger::{cli::NoCliArgs, App, Trigger, TriggerApp};
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

pub struct CronTrigger {
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

impl Trigger for CronTrigger {
    const TYPE: &'static str = "cron";

    type CliArgs = NoCliArgs;

    type InstanceState = ();

    fn new(_cli_args: Self::CliArgs, app: &App) -> anyhow::Result<Self> {
        let cron_components = app
            .trigger_configs::<CronTriggerConfig>(Self::TYPE)?
            .into_iter()
            .map(|(_, config)| Component {
                id: config.component.clone(),
                cron_expression: config.cron_expression.clone(),
            })
            .collect();
        Ok(Self { cron_components })
    }

    fn run(
        self,
        trigger_app: TriggerApp<Self>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        let components = self.cron_components;
        Self::init_cron_scheduler(trigger_app.into(), components)
    }
}

impl CronTrigger {
    async fn init_cron_scheduler(
        engine: Arc<TriggerApp<Self>>,
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
    trigger_app: Arc<TriggerApp<CronTrigger>>,
    component: Component,
}

impl CronEventProcessor {
    fn new(trigger_app: Arc<TriggerApp<CronTrigger>>, component: Component) -> Self {
        Self {
            trigger_app,
            component,
        }
    }

    async fn handle_cron_event(&self, metadata: cron::Metadata) -> anyhow::Result<()> {
        // Load the guest...
        let instance_builder = self.trigger_app.prepare(&self.component.id)?;
        let (instance, mut store) = instance_builder.instantiate(()).await?;
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

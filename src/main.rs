use anyhow::{anyhow, Result};
use async_trait::async_trait;
use clap::Parser;
use serde::{Deserialize, Serialize};
use spin_trigger::{
    cli::{NoArgs, TriggerExecutorCommand},
    EitherInstance, TriggerAppEngine, TriggerExecutor,
};
use std::{
    io::IsTerminal,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};

wasmtime::component::bindgen!({
    path: "cron.wit",
    async: true
});

use fermyon::spin_cron::cron_types as cron;

pub(crate) type RuntimeData = ();

type Command = TriggerExecutorCommand<CronTrigger>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(std::io::stderr().is_terminal())
        .init();

    let t = Command::parse();
    t.run().await
}

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
        let components = self.cron_components.clone();
        let engine = Arc::new(self.engine);
        _ = Self::start_cron_loop(engine.clone(), components).await;

        Ok(())
    }
}

impl CronTrigger {
    async fn start_cron_loop(
        engine: Arc<TriggerAppEngine<Self>>,
        components: Vec<Component>,
    ) -> Result<()> {
        let mut sched = JobScheduler::new().await?;
        for component in components {
            let component = component.clone();
            let engine = engine.clone();
            let _ = sched
                .add(Job::new_async(
                    component.clone().cron_expression.as_str(),
                    move |_uuid, _l| {
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
                .await;
        }

        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                println!("Shut down done");
            })
        }));

        _ = sched.start().await;

        // Create a channel to receive Ctrl+C signal
        let (ctrlc_sender, mut ctrlc_receiver) = tokio::sync::mpsc::channel::<()>(1);

        // Spawn a task to listen for Ctrl+C signal
        tokio::spawn(async move {
            signal::ctrl_c().await.unwrap();
            ctrlc_sender.send(()).await.unwrap();
        });

        // Run the cron scheduler and handle Ctrl+C signal
        let mut running = true;
        while running {
            tokio::select! {
                _ = ctrlc_receiver.recv() => {
                    running = false;
                    println!("Ctrl+C detected. Stopping the loop...");
                },
            }
        }
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
        let res = instance.call_handle_cron_event(&mut store, metadata).await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Component {} failed", self.component.id)),
        }
    }
}

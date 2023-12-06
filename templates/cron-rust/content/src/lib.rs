use spin_cron_sdk::{cron_component, Error, Metadata};

#[cron_component]
fn handle_cron_event(metadata: Metadata) -> Result<(), Error> {
    println!("[{}] Hello from a cron component", metadata.timestamp);
    Ok(())
}

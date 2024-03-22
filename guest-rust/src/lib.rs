use spin_cron_sdk::{cron_component, Error, Metadata};
use spin_sdk::variables;

#[cron_component]
async fn handle_cron_event(metadata: Metadata) -> Result<(), Error> {
    let key = variables::get("something").unwrap_or_default();
    println!(
        "[{}] Hello this is me running every {}",
        metadata.timestamp, key
    );
    Ok(())
}

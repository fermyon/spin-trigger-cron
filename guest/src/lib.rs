wit_bindgen::generate!({
    world: "spin-cron",
    path: "..",
    exports: {
        world: Cron
    }
});
use fermyon::spin_cron::cron_types as cron;
use spin_sdk::variables;

struct Cron;

impl Guest for Cron {
    fn handle_cron_event(metadata: cron::Metadata) -> Result<(), cron::Error> {
        let key = variables::get("something").unwrap_or_default();
        println!(
            "[{}] Hello this is me running every {}",
            metadata.timestamp, key
        );
        Ok(())
    }
}

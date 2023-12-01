pub use spin_cron_macro::*;

pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "spin-cron-sdk",
        path: "..",
    });
}

pub use wit::fermyon::spin_cron::cron_types::Error;
pub use wit::fermyon::spin_cron::cron_types::Metadata;

pub use wit_bindgen;

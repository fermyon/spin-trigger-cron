pub use spin_cron_macro::cron_component;

#[doc(hidden)]
pub use spin_executor as executor;

#[doc(hidden)]
pub mod wit {
    #![allow(missing_docs)]

    wit_bindgen::generate!({
        world: "spin-cron-sdk",
        path: "..",
    });
}

#[doc(inline)]
pub use wit::fermyon::spin_cron::cron_types::Error;
#[doc(inline)]
pub use wit::fermyon::spin_cron::cron_types::Metadata;

#[doc(hidden)]
pub use wit_bindgen;

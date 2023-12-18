# Experimental Cron trigger for Spin

## Install from release

```bash
spin plugins install --url https://github.com/fermyon/spin-trigger-cron/releases/download/canary/trigger-cron.json
```

## Build From source

You will need Rust and the `pluginify` plugin (`spin plugins install --url https://github.com/itowlson/spin-pluginify/releases/download/canary/pluginify.json`).

```bash
cargo build --release
spin pluginify --install
```

## Test

You will need to build and run the spin components. Change the cron expression if needed.

```bash
cd guest
spin build --up
```


## Installing the template 

You can install the template using the following command:

```bash
spin templates install --git https://github.com/fermyon/spin-trigger-cron
```

Once the template is installed, you can create a new application using:

```bash
spin new -t cron-rust hello_cron --accept-defaults
```

To run the newly created app:

```
cd hello_cron
spin build --up
```

## Trigger Configuration

The trigger type is `cron` and there are no application-level configuration options. 

The following options are available to set in the [[trigger.cron]] section:

| Name                  | Type             | Required? | Description |
|-----------------------|------------------|-----------|-------------|
| `component`           | string or table  | required  | The component to run on the schedule given in `cron_expression`. (This is the standard Spin trigger component field.) |
| `cron_expression`     | string           | required  | The `cron` expresison describing the schedule on which to execute the component. |

## Building Cron Components

Currently only a Rust SDK is supported for guest components. The `spin-cron-sdk` along with the [`spin_sdk` crate](https://docs.rs/spin-sdk) can be used to build cron components. The guest code must have a function decorated with the `#[cron_component]` macro. See `guest/src/lib.rs` for an example in rust. 

The signature of the function must be `fn handle_cron_event(metadata: Metadata) -> Result<(), Error>`.

The `Metadata` object contains a single field `timestamp` which contains the duration since 00:00:00 UTC on 1 January 1970 (the Unix epoch) in seconds.
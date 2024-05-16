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

The trigger type is `cron`, and there are no application-level configuration options. 

The following options are available to set in the [[trigger.cron]] section:

| Name                  | Type             | Required? | Description |
|-----------------------|------------------|-----------|-------------|
| `component`           | string or table  | required  | The component to run on the schedule given in `cron_expression`. (This is the standard Spin trigger component field.) |
| `cron_expression`     | string           | required  | The `cron` expression describes the schedule for executing the component. |

### Cron Expression Fields

The `cron_expression` format for scheduling is as follows:

```text
sec   min   hour   day of month   month   day of week   year
*     *     *      *              *       *             *
```

### Cron Expression Characters

The `*` indicates that every value applies; i.e., if `sec` is set to `*`, then every second will trigger execution.
The `/` indicates increments. For example, if `sec` is set to `0/15`, then starting at `0`, the trigger will be executed every 15 seconds.
The `,` separates values. For example, if `sec` is set to `2,8`, then the trigger will execute only on the 2nd and 8th seconds of every minute.
The `-` indicates range, i.e., if the `sec` is set to `5-10`, then the trigger will execute only on the 5th, 6th, 7th, 8th, 9th, and 10th seconds of each minute.
The `0` indicates no execution. If the `sec` is set to `0`, then the trigger can only execute on higher field values such as `min`, `hour`, etc. The lowest second increment is 60 (one minute).

Here is one example that combines a few of the fields mentioned above:

```text
sec   min   hour   day of month   month   day of week   year
0     1/2   11,12  5-10           *       *             *
```

The above schedule of `0 1/2 11,12 5-10 * * *` will execute on the first minute and every subsequent 2 minutes during the 11th hour and the 12 hour (noon) on days 5 through 10 of every month (regardless of the day of the week) and this will repeat through every year.

## Building Cron Components

Currently only a Rust SDK is supported for guest components. The `spin-cron-sdk` along with the [`spin_sdk` crate](https://docs.rs/spin-sdk) can be used to build cron components. The guest code must have a function decorated with the `#[cron_component]` macro. See `guest/src/lib.rs` for an example in rust. 

The signature of the function must be `fn handle_cron_event(metadata: Metadata) -> Result<(), Error>`.

The `Metadata` object contains a single field `timestamp` which contains the duration since 00:00:00 UTC on 1 January 1970 (the Unix epoch) in seconds.

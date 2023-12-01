# Very Experimental Cron trigger for Spin

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

# Robocup Base Station

## TLDR

This Repository houses the code for our raspberry pi base station that relays information from the base computer to the robots.

## Running

To run the base station use one of the following commands

```sh
cargo build --release
./target/release/robocup-base-station "{Base Computer Address}" "{Base Computer Listening Address}"
```

```sh
cargo run --release -- "{Base Computer Address}" "{Base Computer Listening Address}"
```

## Problem

In the past, Robojackets has used a complicated Ubiquity setup for communications with our robots.  This setup included a router, a switch, a cloud key, ... and was incredibly temperamental while still giving us terrible latency.  Therefore, we are switching from a WiFi communication setup to a rf radio based communication to hopefully reduce the latency and the pain that is setting up our competition setup.

## Docs

To read documentation for the Repo run the following command:

```sh
cargo doc --open
```

## Useful Tips

### VSCode - Rust-Analyzer - Config

When using rust-analyzer and cross compiling this project to the raspberry pi include the following in the .vscode/settings.json file

```json
{
    "rust-analyzer.cargo.target": "armv7-unknown-linux-gnueabihf",
}
```
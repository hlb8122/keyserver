# Cash:web Keyserver (Rust Implementation)
[![Build Status](https://travis-ci.org/hlb8122/keyserver-rust.svg?branch=master)](https://travis-ci.org/hlb8122/keyserver-rust)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[**Golang Implementation**](https://github.com/cashweb/keyserver/)

This repository hosts a reference implementation of the Cash:web Keyserver protocol. The goal is to provide a distributed, simple-to-use and cryptographically verifiable way to look up xpubkeys, and other metadata, from their hashes. The hashes are commonly available within Bitcoin Cash Addresses such as *bitcoincash:pqkh9ahfj069qv8l6eysyufazpe4fdjq3u4hna323j*.

## Why not existing systems?

Traditional keyservers are subject to certificate spamming attacks. By being a first-class citizen in the cryptocurrency ecosystem, we are able to charge for key updates. This prevents an explosion of advertised certificates, and provides some funding for node operators. Other systems like OpenAlias, require that you trust the service provider is providing the correct addresses, while this keyserver cannot forge such updates as they are tied to a keyid which has been provided via another channel. At most, a malicious keyserver can censor a particular key, in which case other keyservers will provide it.

## Running a Server

### Setting up Bitcoin

Bitcoin must be run with [RPC](https://bitcoin.org/en/developer-reference#remote-procedure-calls-rpcs) and raw transaction [ZMQ](https://github.com/bitcoin/bitcoin/blob/master/doc/zmq.md) enabled.

### Setting up a Payment Server

See installation instructions [here](https://github.com/hlb8122/payment-server).

### Build

Install [Rust](https://www.rust-lang.org/tools/install) then

```bash
sudo apt install -y clang pkg-config libssl-dev libzmq3-dev
cargo build --release
```

The executable will be located at `./target/release/keyserver`.

### Configuration

Settings may be given by `JSON`, `TOML`, `YAML`, `HJSON` and `INI` files and, by default, are located at `~/.keyserver-rust/config.*`.

| Name | Description | Default |
| - | - | - |
| `bind` | Bind address | `127.0.0.1:8080` |
| `zmq_addr` | Bitcoin ZMQ address | `tcp://127.0.0.1:28332` |
| `secret` | Keyserver secret | `secret` |
| `db_path` | Database path | `~/.keyserver-rust/db` |
| `payment_server_url` | Payment server URL | `http://127.0.0.1:8900` | 
| `network` | Bitcoin network | `regnet` |

The `network` parameter must be either `mainnet`, `testnet` or `regnet`.

Each of the parameters above can be overloaded via command line (replacing `_` with `-`). Additionaly, `--config` can be passed via command line to specify a configuration file at a custom location.

A full list of command line arguments can be viewed via `keyserver --help`.

### Running

```bash
./target/release/keyserver [OPTIONS]
```

Alternatively, copy `./static/` folder and `keyserver` to a directory and run `keyserver` from there.

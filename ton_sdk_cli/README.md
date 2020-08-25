# TON SDK command line tool

`tonsdk` is a command line interface utility designed to work with TON SDK.

## How to build

```bash
cargo build [--release]
```

## How to test
```bash
cargo test
```

## How to run

```bash
> tonsdk [-n network] command parameters
```
Where `network` is a network address. By default, `tonsdk` connects to `net.ton.dev` network.


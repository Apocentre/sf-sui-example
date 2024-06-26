# substreams-sui-example
This repository show cases consumption of an Sui Substreams. More specifically, we will index all Coins created on Sui mainnet.

Build and package hanlers
===

From the root

```bash
 cargo build -p sf-modules --target wasm32-unknown-unknown --release
 substreams pack sf-modules/substreams.yaml
```

Move the created package to `coin-sink`

```bash
mv sf-modules/sf-modules-v0.1.0.spkg coin-sink/
```

Start `firesui` as described in [here](https://github.com/apocentre/firehose-sui)

Finally start the `coin-sink`:

```bash
cd coin-sink
ENV=development cargo run
```

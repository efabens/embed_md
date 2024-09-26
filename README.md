# Embed MD

## Usage

```shell
cargo build --release

target/release/embed_md --help
```

Embed_md will process an entire file if no id is provided
```shell
embed_md path/to/file.md
```
A single embed can be processed by providing an id
```shell
embed_md path/to/file.md --id my_embed
```

see the [samples](./samples) directory for examples (this is not comprehensive)

## TODO

- Have cache dependent on parent hash, it probably needs cycle detection at that point as well
- Global envs
- default file wide configs (specifically for caching, often I want to use hash everywhere except where I don't)

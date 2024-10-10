# Contributing guidelines

## Testing

Snapshot tesing is done using `insta`.
For more information, see [insta.rs/docs](https://insta.rs/docs) and
the [Getting Started](https://insta.rs/docs/quickstart/#installation) guide.

To review snapshots during development, it is recommended to use `cargo-insta`
which you can install by running:

```sh
cargo install cargo-insta
```

then you can use it by running:

```sh
cargo insta review
```

For more information, see [Cargo Insta](https://insta.rs/docs/cli/).

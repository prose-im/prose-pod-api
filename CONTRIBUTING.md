# Contributing guidelines

## How to contribute?

This project is still in its very early stage. Until it reaches release 1.0, please do not send contributions as they will probably slow us down more than anything.

After release 1.0, contributions will be more than welcome though!

## Tools you need

```bash
cargo install sea-orm-cli
```

To use `prosodyctl` locally, you need `prosodyctl` and its dependencies.

```bash
brew install prosodyctl
luarocks install luaunbound
```

## Testing

As explained in [ADR: Write tests with the Gherkin syntax](./ADRs/2024-01-11-a-write-tests-in-gherkin.md),
we are using Gherkin and Cucumber to run tests. Therefore, you can use this command to run the tests:

```bash
cargo test --test cucumber
```

You could also run `cargo test` but it runs unit tests in `src/`, which we don't need.

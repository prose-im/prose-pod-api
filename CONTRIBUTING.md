# Contributing guidelines

## How to contribute?

This project is still in its very early stage. Until it reaches release 1.0, please do not send contributions as they will probably slow us down more than anything.

After release 1.0, contributions will be more than welcome though!

## Tools you need

### `sea-orm-cli`

If you work on databse migrations, you will probably need `sea-orm-cli`:

```bash
cargo install sea-orm-cli
```

### `prosodyctl`

To use `prosodyctl` locally, you need `prosodyctl` and its dependencies.

```bash
brew install prosodyctl
luarocks install luaunbound
```

## Updating dependencies

```bash
rustup upgrade && cargo update
make update-redoc
```

## Testing

As explained in [ADR: Write tests with the Gherkin syntax](./ADRs/2024-01-11-a-write-tests-in-gherkin.md),
we are using Gherkin and Cucumber to run tests. Therefore, you can use this command to run the tests:

```bash
cargo test --test behavior
```

You could also run `cargo test` but it runs unit tests in `src/`, which we don't need.

> [!TIP]
> While developing a feature, add a `@testing` tag to a `Feature`, `Rule` or `Scenario` (non-exhaustive)
> and then use `cargo test --test behavior -- --tags '@testing'` to run only matching `Scenario`s.

## Building the Docker image

```bash
docker build -t proseim/prose-pod-api .
```

To build the API in debug mode (e.g. to use predictable data generators),
you can use the `CARGO_INSTALL_EXTRA_ARGS` [Docker `ARG`]:

```bash
docker build -t proseim/prose-pod-api --build-arg CARGO_INSTALL_EXTRA_ARGS='--debug' .
```

[Docker `ARG`]: https://docs.docker.com/reference/dockerfile/#arg "Dockerfile reference | Docker Docs"

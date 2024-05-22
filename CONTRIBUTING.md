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

### Smoke testing

As explained in [ADR: Write tests with the Gherkin syntax](./ADRs/2024-01-11-a-write-tests-in-gherkin.md),
we are using Gherkin and Cucumber to run tests. Therefore, you can use this command to run the tests:

```bash
cargo test --test behavior
```

You could also run `cargo test` but it runs unit tests in `src/`, which we don't need.

> [!TIP]
> While developing a feature, add a `@testing` tag to a `Feature`, `Rule` or `Scenario` (non-exhaustive)
> and then use `cargo test --test behavior -- --tags '@testing'` to run only matching `Scenario`s.

### Integration testing

#### Installing dependencies

For integration tests, we use [Artillery](https://www.artillery.io/). To install it, follow instructions at [Get Artillery – Artillery Docs](https://www.artillery.io/docs/get-started/get-artillery) or just run the following command if you don't have an exotic setup:

```bash
npm install -g artillery@latest
```

Tests need an API key to run correctly, and since it expires we can't have a hard-coded test key. To allow generating JWTs from the testing script, you will need to install [`jwt-cli`](https://github.com/mike-engel/jwt-cli). For this, you can follow [`jwt-cli`'s installation instructions](https://github.com/mike-engel/jwt-cli?tab=readme-ov-file#installation) or just run:

```bash
cargo install jwt-cli
```

#### Setting up environment

You also need to clone [prose-pod-server](https://github.com/prose-im/prose-pod-server) and [prose-pod-system](https://github.com/prose-im/prose-pod-system). Once that done, you will need to set environment variables so our testing script can pick up the locations:

```bash
export PROSE_POD_API_DIR=???
export PROSE_POD_SYSTEM_DIR=???
```

Finally, since integration tests run on final containers, you have to build `prose-pod-server` and `prose-pod-api`:

```bash
docker build -t proseim/prose-pod-api "${PROSE_POD_API_DIR:?}"
PROSE_POD_SERVER_DIR=???
docker build -t proseim/prose-pod-server "${PROSE_POD_SERVER_DIR:?}"
```

#### Running tests

Then, run the tests using:

```bash
make integration-test
```

If you need to debug, you might need to see HTTP requests and responses. The good thing is that you can simply use Artillery's `DEBUG` environment variable as documented in [Debugging – HTTP Engine – Artillery Docs](https://www.artillery.io/docs/reference/engines/http#debugging). The most verbose version (not _that_ verbose actually) is:

```bash
DEBUG=http* make integration-test
```

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

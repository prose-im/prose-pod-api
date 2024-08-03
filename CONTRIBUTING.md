# Contributing guidelines

## How to contribute?

This project is still in its very early stage. Until it reaches release 1.0, please do not send contributions as they will probably slow us down more than anything. Before the 1.0 release, files will be cleaned up, moved around and refactored. Until then it will be quite messy because of very frequent substantial changes.

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

After you have setup your environment to run smoke tests and integration tests, you can run all of them in a single command using:

```bash
make test
```

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

For integration tests, we use [Step CI]. To install it, follow instructions at [Getting started | Step CI Docs](https://docs.stepci.com/guides/getting-started.html) or just run the following command if you don't have an exotic setup:

```bash
npm install -g stepci
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

If a test fails, Step CI will automatically print some additional information to help you debug the issue. We also print container logs so you can see internal errors.

> [!TIP]
> Step CI collects analytics when it's used, which is fine, but it also means `stepci` will fail if it can't reach the analytics server.
> If you need to run tests offline, run `export STEPCI_DISABLE_ANALYTICS=true` before running the tests.

## Building the Docker image

To build the Docker image, you can use the helper script (which builds the image as `proseim/prose-pod-api:latest`):

```bash
./scripts/build-image.sh TARGET_ARCH
```

> [!TIP]
> For Apple Silicon Macs, that's `./scripts/build-image.sh aarch64-apple-darwin`. For Intel Macs, it would be `./scripts/build-image.sh x86_64-unknown-linux-musl`.

To build the API in debug mode (e.g. to use predictable data generators), you can use the `--debug` argument:

```bash
./scripts/build-image.sh TARGET_ARCH --debug
```

[Step CI]: https://stepci.com/ "Step CI homepage"

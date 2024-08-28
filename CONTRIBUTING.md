# Contributing guidelines

## How to contribute?

This project is still in its very early stage. Until it reaches release 1.0, please do not send contributions as they will probably slow us down more than anything. Before the 1.0 release, files will be cleaned up, moved around and refactored. Until then it will be quite messy because of very frequent substantial changes.

After release 1.0, contributions will be more than welcome though!

## Tools you need

### `task`

Instead of using [GNU Make], we are using [Task] for its simplicity and flexibility. You can find installation instructions on [taskfile.dev/installation](https://taskfile.dev/installation/), or just run the folowing on macOS:

```bash
brew install go-task
```

To list all available commands, use:

```bash
task -a
```

### `cross`

To build the Docker image locally, we need to cross-compile Rust code for a different architecture. To avoid cluttering your local environment, we use [`cross`] which handles everything transparently. You can find installation instructions on [github.com/cross-rs/cross#installation](https://github.com/cross-rs/cross?tab=readme-ov-file#installation), or just run the folowing:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### `sea-orm-cli`

If you work on database migrations, you will probably need `sea-orm-cli`:

```bash
cargo install sea-orm-cli
```

## Updating dependencies

```bash
task update
```

## Testing

After you have setup your environment to run smoke tests and integration tests, you can run all of them in a single command using:

```bash
task test
```

This has the added benefit of updating the version of Rust used to run the tests in [`README.md`](./README.md).

### Smoke testing

```bash
task smoke-test
```

As explained in [ADR: Write tests with the Gherkin syntax](./ADRs/2024-01-11-a-write-tests-in-gherkin.md), we are using Gherkin and Cucumber to run the tests. Therefore, you won't be able to filter tests using `cargo test`. To do so, add a `@testing` tag to a `Feature`, `Rule` or `Scenario` (non-exhaustive) and then use `task smoke-test -- --tags '@testing'` to run only matching `Scenario`s.

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
# Build the Prose Pod API image.
task build-image -- --debug
# Build the Prose Pod Server image.
PROSE_POD_SERVER_DIR=???
docker build -t proseim/prose-pod-server:latest "${PROSE_POD_SERVER_DIR:?}"
```

#### Running tests

Then, run the tests using:

```bash
task integration-test
```

If a test fails, Step CI will automatically print some additional information to help you debug the issue. We also print container logs so you can see internal errors.

> [!TIP]
> Step CI collects analytics when it's used, which is fine, but it also means `stepci` will fail if it can't reach the analytics server.
> If you need to run tests offline, run `export STEPCI_DISABLE_ANALYTICS=true` before running the tests.

## Building the Docker image

To build the Docker image, you can use the helper script (which builds the image as `proseim/prose-pod-api:latest`):

```bash
task build-image [-- [TARGET_ARCH] [--debug] [--help]]
```

If you don't set `TARGET_ARCH`, `build-image` will build `proseim/prose-pod-api:latest` for your local architecture. If you set `TARGET_ARCH`, `build-image` will build `proseim/prose-pod-api:latest` for the desired architecture. Since the image runs Alpine Linux, which likely doesn't match your local target triple, `build-image` builds the Prose Pod API using [`cross`](#cross). You can set `PROSE_POD_API_IMAGE` to override the final name of the image (useful when cross-compiling).

To build the API in debug mode (e.g. to use predictable data generators), you can use the `--debug` argument:

```bash
task build-image -- [TARGET_ARCH] --debug
```

[Step CI]: https://stepci.com/ "Step CI homepage"
[Task]: https://stepci.com/ "Task"
[GNU Make]: https://www.gnu.org/software/make/ "Make - GNU Project - Free Software Foundation"
[`cross`]: https://github.com/cross-rs/cross "cross-rs/cross: “Zero setup” cross compilation and “cross testing” of Rust crates"

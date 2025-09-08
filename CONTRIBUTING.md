# Contributing guidelines

## How to contribute?

This project is still in its very early stage. Until it reaches release 1.0, please do not send contributions as they will probably slow us down more than anything. Before the 1.0 release, files will be cleaned up, moved around and refactored. Until then it will be quite messy because of very frequent substantial changes.

After release 1.0, contributions will be more than welcome though!

## Tools you need

### `task`

Instead of using [GNU Make], we are using [Task] for its simplicity and flexibility.
You can find installation instructions on [taskfile.dev/installation],
or just run the folowing on macOS:

```bash
brew install go-task
```

To list all available commands, use:

```bash
task -a --sort none
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

As explained in [ADR: Write tests with the Gherkin syntax](./docs/ADRs/2024-01-11-a-write-tests-in-gherkin.md), we are using Gherkin and Cucumber to run the tests. Therefore, you won't be able to filter tests using `cargo test`. To do so, add a `@testing` tag to a `Feature`, `Rule` or `Scenario` (non-exhaustive) and then use `task smoke-test -- --tags '@testing'` to run only matching `Scenario`s.

### Integration testing

#### Installing dependencies

For integration tests, we use [Step CI]. To install it, follow instructions at [Getting started | Step CI Docs](https://docs.stepci.com/guides/getting-started.html) or just run the following command if you don't have an exotic setup:

```bash
npm install -g stepci
```

#### Running tests

Then, run the tests using:

```bash
task integration-test -- --server=local
```

If a test fails, Step CI will automatically print some additional information to help you debug the issue. We also print container logs so you can see internal errors.

> [!TIP]
> Step CI collects analytics when it's used, which is fine, but it also means `stepci` will fail if it can't reach the analytics server.
> If you need to run tests offline, run `export STEPCI_DISABLE_ANALYTICS=true` before running the tests.

## Building the Docker image

To build the Docker image, you can use the helper script (which builds the image as `proseim/prose-pod-api:local`):

```bash
task build-image [-- [--platform=TARGET_PLATFORM] [--profile=CARGO_PROFILE] [--help]]
```

If you don't set `TARGET_PLATFORM`, `build-image` will build `proseim/prose-pod-api:local` for your local platform. If you set `TARGET_PLATFORM`, `build-image` will build `proseim/prose-pod-api:local` for the desired platform. You can set `PROSE_POD_API_IMAGE` to override the final name of the image.

To build the API in debug mode (e.g. to use predictable data generators), you can use the `--profile=dev` argument:

```bash
task build-image -- [--platform=TARGET_PLATFORM] --profile=dev
```

## Style

(This is a work in progress)

- Always parse user input (see [Parse all user input · Issue #164 · prose-im/prose-pod-api](https://github.com/prose-im/prose-pod-api/issues/164)). To make it easier, we use use [validator](https://crates.io/crates/validator) + [serdev](https://crates.io/crates/serdev) to validate during deserialization. When you need this (i.e. when some wrapped type needs validation), derive `serdev::Deserialize` explicitly and do not `use` it. This makes intent clearer and helps spotting forgotten `serde(validate = "Validate::validate")`. More generally, always make `serdev` explicit when it is used; otherwise (e.g. for serialization) you can `use` it.
- Derive `Debug` everywhere.
- Derive `Clone` only when cloning is cheap.

[Step CI]: https://stepci.com/ "Step CI homepage"
[Task]: https://stepci.com/ "Task"
[GNU Make]: https://www.gnu.org/software/make/ "Make - GNU Project - Free Software Foundation"
[taskfile.dev/installation]: https://taskfile.dev/installation/ "Installation | Task"

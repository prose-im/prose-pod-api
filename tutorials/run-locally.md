# How to run a Prose Pod API locally?

When developing the [Prose Pod Dashboard], one needs to easily start a Prose Pod API locally.
This document explains how to do just this.

## First time setup

### Intall the tools you will need

#### `git`

We use Git as version control system so you will need to install it.
You probably have it installed already, but if you don't, have a look at
[Git - Installing Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).

#### `task`

Instead of using [GNU Make], we are using [Task] for its simplicity and flexibility.
You can find installation instructions on [taskfile.dev/installation],
or just run the following on macOS:

```bash
brew install go-task
```

To list all available commands, use:

```bash
task -a
```

#### Docker

We need to run Docker images so you must have Docker installed.
See [Install | Docker Docs](https://docs.docker.com/engine/install/).

### Initialize your environment

To avoid you having to copy-paste tons of commands, we hid all of the logic behind helper scripts
which you can invoke using `task`. The first task to run is `local-init`.

But before that, you must declare where you want the repositories to be
(replace `???` by the desired location):

```sh
export PROSE_POD_API_DIR=???
export PROSE_POD_SYSTEM_DIR=???
```

Then, run:

```sh
git clone https://github.com/prose-im/prose-pod-api.git "${PROSE_POD_API_DIR:?}"
git -C "${PROSE_POD_API_DIR:?}" submodule update --init
task --taskfile "${PROSE_POD_API_DIR:?}"/Taskfile.yaml local-init
```

> [!TIP]
> To avoid having to export the environment variables every time, you can add them to a local `.env` file
> which will be sourced automatically when you run the next commands:
>
> ```sh
> echo "export PROSE_POD_API_DIR='${PROSE_POD_API_DIR:?}'
> export PROSE_POD_SYSTEM_DIR='${PROSE_POD_SYSTEM_DIR:?}'" \
> >> "${PROSE_POD_API_DIR:?}"/paths.env
> ```

> [!TIP]
> To run the `task` commands, you will have to `cd` into the Prose Pod API repository,
> or invoke `task` telling it where to find the Taskfile. If you prefer the latter scenario,
> you can export `PROSE_POD_API_DIR` in your `~/.zprofile` (or equivalent) and call
> `task --taskfile "${PROSE_POD_API_DIR:?}"/Taskfile.yaml <task>` instead of `task <task>`.

## Run the API

At the root of the `prose-pod-api` repository, run:

```sh
task local-run
```

The above command runs the latest released versions, but you can change this behavior:

```sh
# Run latest patches (latest commits, unreleased):
task local-run -- --api=edge
# Run a specific version:
task local-run -- --api=1.2.3
task local-run -- --api=1.2
task local-run -- --api=1
```

## When you want to start fresh with a Prose Pod API that has no data

At the root of the `prose-pod-api` repository, run:

```sh
task local-reset
```

OR do the "Initialize your environment" phase again but with different directory paths.
This way you can have multiple instances of the API with different states.

[Prose Pod Dashboard]: https://github.com/prose-im/prose-pod-dashboard "prose-im/prose-pod-dashboard: Prose Pod dashboard. Static Web application used to interact with the Prose Pod API."
[Task]: https://stepci.com/ "Task"
[GNU Make]: https://www.gnu.org/software/make/ "Make - GNU Project - Free Software Foundation"
[taskfile.dev/installation]: https://taskfile.dev/installation/ "Installation | Task"

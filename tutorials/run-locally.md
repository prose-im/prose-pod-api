# How to run a Prose Pod API locally?

When developing the [Prose Pod Dashboard], one needs to easily start a Prose Pod API locally.
This document explains how to do just this.

## First time setup

### Install the tools you will need

#### `git`

We use Git as version control system so you will need to install it.
You probably have it installed already, but if you don’t, have a look at
[Git - Installing Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).

#### `task`

To avoid you having to copy-paste tons of commands, we hid all of the logic behind helper scripts.

Instead of using [GNU Make], we are using [Task] for its simplicity and flexibility.
You can find installation instructions on [taskfile.dev/installation],
or just run the following on macOS:

```bash
brew install go-task
```

To list all available commands, use:

```bash
task -a --sort none
```

#### Docker

We need to run Docker images so you must have Docker installed.
See [Install | Docker Docs](https://docs.docker.com/engine/install/).

### Clone the repository

But before that, you must declare where you want the repositories to be
(replace `???` by the desired location):

```sh
PROSE_POD_API_DIR=???
git clone https://github.com/prose-im/prose-pod-api.git "${PROSE_POD_API_DIR:?}"
git -C "${PROSE_POD_API_DIR:?}" submodule update --init
cd "${PROSE_POD_API_DIR:?}"
```

## Run the API

At the root of the `prose-pod-api` repository, run:

```sh
task local:run
```

The above command runs the latest released versions, but you can change this behavior:

```sh
# Run latest patches (latest commits, unreleased):
task local:run -- --api=edge
# Run a specific version:
task local:run -- --api=1.2.3
task local:run -- --api=1.2
task local:run -- --api=1
```

### Check sent emails

When inviting someone to a workspace, the invitation accept and reject links are only sent
via email and there is no way to retrieve it from the API. The good news is `task local:run`
also provides a mocked SMTP server which exposes a Web UI where you can access all emails.
It is available at <http://localhost:8025/>.

### Check the logs

To see the logs of your **last** local run, you can run:

```sh
# See the logs from all services:
task local:logs
# See only the logs from the API or XMPP server:
task local:logs -- api server
```

> [!WARNING]
> Logs are not saved from one run to the next, so make sure to copy them
> before starting a new instance.

### Create pre-populated environments

Having only one instance of the API is good to play around, but when developing software on top
of the Prose Pod API (like the Prose Pod Dashboard), you might want to save a snaphot of the API
and XMPP server data so you can start again from this point at a later time.

For those situations, we created a concept of “scenarios” which you can interact from.
This section will list use cases along with the commands you would use in that situation.

> [!WARNING]
> Your scenarios won’t be committed, so make sure to back them up
> before deleting the repository!

#### Use case: “I’m in a state I’d like to reuse later”

When you feel like you’re in a good state in some scenario and would like to “snapshot” it,
you can run:

```sh
# Based on the default scenario, which is the one you run when doing `task local:run`:
task local:scenarios:create -- foo
# Based on an existing scenario:
task local:scenarios:create -- bar --based-on=foo
```

#### Use case: “I’d like to start from a scenario I created previously”

If you want to start from an existing scenario for a single run, use:

```sh
task local:run -- --scenario=foo --ephemeral
```

You can also derive a new environment from this scenario using:

```sh
task local:scenarios:create -- bar --based-on=foo
task local:run -- --scenario=bar
```

#### Use case: “I’d like to start again from scratch”

```sh
# Reset the state of the default scenario:
task local:reset
# Reset the state of a single scenario (this won’t affect scenarios derived from it):
task local:scenarios:reset -- foo
# Reset the state of multiple scenarios at once (this won’t affect scenarios derived from it):
task local:scenarios:reset -- foo bar
```

#### Use case: “I don’t remember the names of the scenarios I created”

```sh
task local:scenarios:list
```

#### Use case: “There is a scenario I don’t need anymore, I’d like to delete it”

```sh
# Delete a single scenario (this won’t delete scenarios derived from it):
task local:scenarios:delete -- foo
# Delete multiple scenarios at once (this won’t delete scenarios derived from it):
task local:scenarios:delete -- foo bar
```

[Prose Pod Dashboard]: https://github.com/prose-im/prose-pod-dashboard "prose-im/prose-pod-dashboard: Prose Pod dashboard. Static Web application used to interact with the Prose Pod API."
[Task]: https://taskfile.dev/ "Task homepage"
[GNU Make]: https://www.gnu.org/software/make/ "Make - GNU Project - Free Software Foundation"
[taskfile.dev/installation]: https://taskfile.dev/installation/ "Installation | Task"

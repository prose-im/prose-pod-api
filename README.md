# prose-pod-api

[![Test](https://github.com/prose-im/prose-pod-api/actions/workflows/test.yaml/badge.svg?branch=master)](https://github.com/prose-im/prose-pod-api/actions/workflows/test.yaml)

**Prose Pod API server. REST API used for administration and management.**

Copyright 2022-2025, Prose Foundation - Released under the [Mozilla Public License 2.0](./LICENSE.md).

_Tested at Rust version: `rustc 1.89.0 (29483883e 2025-08-04)`_

## Quick Start

### Run the API server

The API can be ran with the following command:

```sh
task local:run -- --scenario=demo --ephemeral --api=edge
```

This will run the _latest_ API version, with the _demo scenario_, in _ephemeral mode_. This means that no data will be persisted. Every time you restart the API, you will start from the same fresh demo data again.

The API will be running at [localhost:8080](http://localhost:8080).

### Read the API documentation

To start the API documentation server, run the following command:

```sh
task openapi:preview-docs
```

Then, open the documentation at [localhost:8081](http://localhost:8081).

## License

Licensing information can be found in the [LICENSE.md](./LICENSE.md) document.

## :fire: Report A Vulnerability

If you find a vulnerability in any Prose system, you are more than welcome to report it directly to Prose Security by sending an encrypted email to [security@prose.org](mailto:security@prose.org). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production systems running an unpatched version.

**:warning: You must encrypt your email using Prose Security GPG public key: [:key:57A5B260.pub.asc](https://files.prose.org/public/keys/gpg/57A5B260.pub.asc).**

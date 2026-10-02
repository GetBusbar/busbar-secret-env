<!-- fleet:header:begin (rendered by `busbar-release plugin sync` from GetBusbar/busbar-release template/ and busbar's plugins.yaml; edit it there) -->
# busbar-secret-env

First-party signed kind:secret plugin cdylib: the env secret source (module: env), resolving a config secret from the process environment. Part of the default build; drop the signed tarball into plugins/ for a bare-bones build.

| kind | alias | crate | busbar | license |
|---|---|---|---|---|
| `secret` | `env` | `busbar-secret-env-plugin` | 1.6.0 (pinned in `.busbar-ref`) | MIT |

[![ci](https://github.com/GetBusbar/busbar-secret-env/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/GetBusbar/busbar-secret-env/actions/workflows/ci.yml)
<!-- fleet:header:end -->

## What it is for

`busbar-secret-env` is a `kind: secret` busbar plugin.

## Config

Configured under the `env` module name.

## Build

```bash
cargo build --release -p busbar-secret-env-plugin
```

## Tests

```bash
cargo test --workspace --locked
```

## License

Apache-2.0. See [LICENSE](LICENSE).

<!-- fleet:header:begin (rendered by `busbar-release plugin heal` from GetBusbar/busbar-release template/ and busbar's plugins.yaml; edit it there) -->
# busbar-secret-env

First-party signed kind:secret plugin cdylib: the environment-variable secret source (`{ env: VAR }`), packaged as a droppable busbar plugin. busbar links it in the default build; a build without it resolves `{ env: VAR }` only when the signed tarball is dropped into plugins/.

| kind | alias | crate | busbar | license |
|---|---|---|---|---|
| `secret` | `env` | `busbar-secret-env-plugin` | 1.6.0 (pinned in `.busbar-ref`) | Apache-2.0 |

[![ci](https://github.com/GetBusbar/busbar-secret-env/actions/workflows/ci.yml/badge.svg?branch=dev)](https://github.com/GetBusbar/busbar-secret-env/actions/workflows/ci.yml)
<!-- fleet:header:end -->

## What it is for

`busbar-secret-env` is a `kind: secret` busbar plugin. It resolves a secret reference `{ env: VAR }`
to the bytes of the environment variable `VAR`, exactly as stored (untrimmed).

It is linked in busbar's default build and also ships as a droppable cdylib for builds without it.
It reads only the process environment: no files, no network.

## Config

Configured under the `env` module name. A secret reference is `{ env: MY_VAR }`, the sugar for
`{ module: env, settings: { key: ... } }`; it resolves to the environment variable named by `key`, byte for byte (untrimmed).

A reference names `key` and nothing else: a missing or blank `key` fails INVALID, and so does
any other setting, named in the refusal. An unset or non-UTF-8 variable fails NOT_FOUND ("is unset"); an empty one fails INVALID. The value is read into a wiped buffer, and a refused value is wiped as it drops. No refusal carries the secret.

The source takes no instance settings (`secrets.env.settings`): none or `{}` opens it, and any
setting is refused naming it, at `validate`, `open` and `refresh` alike.

## Build

```bash
cargo build --release -p busbar-secret-env-plugin --features dropped-in
```

`dropped-in` puts the `busbar_plugin_door` symbol in the cdylib; a build that links the logic
crate as a row carries no door symbol.

## Tests

```bash
cargo test --workspace --locked
```

## License

Apache-2.0. See [LICENSE](LICENSE).

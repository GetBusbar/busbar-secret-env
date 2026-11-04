// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! THE `env` SECRET SOURCE for busbar's `kind: secret` plugin family, on the secret kind's memory
//! ABI (`busbar_contract::abi::secret`): the logic and its door ([`door::door`]).

#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, deny(unsafe_code))]

use busbar_contract::abi::sdk::cred::Held;
use busbar_contract::secret::{SecretModuleError, SecretResult};

pub mod door;

/// The word config names this source by: its module alias and its reference sugar (`{ env: X }`).
pub const WORD: &str = "env";

/// The settings key the sugar's value lands under (`{ env: X }` → `settings.key`).
pub const KEY: &str = "key";

/// Where a reference names its value, as a refusal of instance settings says it (the SDK's
/// `takes_no_settings`, at `validate`, `open` and `refresh` alike).
pub const NAMES: &str = "variable in its own settings.key";

/// The plugin's name (the manifest name).
pub const NAME: &str = "busbar-secret-env";

/// The plugin's `declares` section (its `declares.json`): it reads only the process environment, so
/// it declares nothing.
pub const DECLARES: &str = include_str!("../declares.json");

/// Resolve `settings` (`{ key: VAR }`) to the variable's value, byte for byte, in a zeroizing
/// [`Held`] that reaches the host only through the SDK's `resolve` answer.
///
/// # Errors
/// INVALID for a missing, blank, `=`/NUL-containing or extra setting, a non-UTF-8 value or an
/// empty value; NOT_FOUND for an unset variable. No refusal carries the value: a name is shown
/// escaped, and a `key` is shown only up to its first `=` or NUL.
pub fn resolve(
    settings: &serde_json::Map<String, serde_json::Value>,
) -> SecretResult<Held<Vec<u8>>> {
    let var = reference_key(settings)?;
    resolve_variable(var)
}

pub(crate) fn reference_key(
    settings: &serde_json::Map<String, serde_json::Value>,
) -> SecretResult<&str> {
    if let Some(other) = settings.keys().find(|k| k.as_str() != KEY) {
        let other = other.escape_debug();
        return Err(SecretModuleError::invalid(format!(
            "secret module 'env' does not take settings.{other}; a reference names only \
             settings.key (fail-closed)"
        )));
    }
    let var = match settings.get(KEY).and_then(|v| v.as_str()) {
        Some(v) if !v.trim().is_empty() => v,
        _ => {
            return Err(SecretModuleError::invalid(
                "secret module 'env' requires settings.key naming the environment variable \
                 (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)",
            ))
        }
    };
    if let Some(at) = var.find(['=', '\0']) {
        let name = var[..at].escape_debug();
        let stop = if var[at..].starts_with('=') {
            "="
        } else {
            "\\0"
        };
        return Err(SecretModuleError::invalid(format!(
            "secret module 'env': settings.key '{name}{stop}...' is not a valid environment \
             variable name (contains '=' or NUL; fail-closed)"
        )));
    }
    Ok(var)
}

fn resolve_variable(name: &str) -> SecretResult<Held<Vec<u8>>> {
    let var = name.escape_debug();
    let unset = || {
        SecretModuleError::not_found(format!(
            "secret env:{var} cannot resolve: environment variable '{var}' is unset"
        ))
    };
    let Some(value) = std::env::var_os(name) else {
        return Err(unset());
    };
    let bytes = value.into_encoded_bytes();
    #[cfg(test)]
    wiping_tests::watch(&bytes);
    let text = std::str::from_utf8(&bytes).is_ok();
    let held = Held::new(bytes);
    if !text {
        return Err(SecretModuleError::invalid(format!(
            "secret env:{var} cannot resolve: environment variable '{var}' is not valid UTF-8"
        )));
    }
    if held.is_empty() {
        return Err(SecretModuleError::invalid(format!(
            "secret env:{var} resolved to an EMPTY value; a secret must be non-empty \
             (fail-closed)"
        )));
    }
    Ok(held)
}

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/wiping_tests.rs"]
mod wiping_tests;

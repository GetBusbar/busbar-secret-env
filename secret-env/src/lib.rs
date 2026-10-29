// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! THE `env` SECRET SOURCE for busbar's `kind: secret` plugin family, on the secret kind's memory
//! ABI (`busbar_contract::abi::secret`). This crate is the LOGIC and its door ([`door::door`]); the
//! dropped-in `cdylib` is the sibling `busbar-secret-env-plugin` crate, which exports the same door
//! as `busbar_plugin_door`.
//!
//! `resolve` reads the environment variable its settings' `key` names. `{ env: VAR }` is the
//! reference sugar for `{ module: env, settings: { key: VAR } }`.
//!
//! It is an ordinary plugin: linked in the default build or dropped in, called through the secret
//! kind table by the one dispatcher, and loaded only when a reference names it. A variable or `key`
//! is refused with `is unset`, `EMPTY value` or `requires settings.key`. Also refused: settings
//! bytes that are not a JSON object, a reference naming a setting other than `key`, and instance
//! settings, which this source takes none of (the SDK's `takes_no_settings`, naming [`NAMES`]).
//! None carries the value.
//!
//! THE VALUE IS HELD, NEVER COPIED (the credential chip, row 6: host-owned secret bytes): it is
//! read as bytes straight into a zeroizing [`Held`], refused or not, and leaves it only through the
//! SDK's `resolve` answer, into the host's buffer ([`door`]). A refused value (empty, or not
//! UTF-8) is wiped as it drops, and so is a resolved one once the host has it.

#![forbid(unsafe_code)]

use busbar_contract::abi::sdk::cred::Held;
use busbar_contract::secret::{SecretModuleError, SecretResult};

pub mod door;

/// The word config names this source by: its module alias and its reference sugar
/// (`{ env: X }`), spelled here, never by the contract (busbar step 2c).
pub const WORD: &str = "env";

/// The settings key the sugar's value lands under (`{ env: X }` → `settings.key`).
pub const KEY: &str = "key";

/// Where a reference names its value, as a refusal of instance settings says it (the SDK's
/// `takes_no_settings`, at `validate`, `open` and `refresh` alike).
pub const NAMES: &str = "variable in its own settings.key";

/// The plugin's name (the manifest name).
pub const NAME: &str = "busbar-secret-env";

/// Resolve `settings` (`{ key: VAR }`) to the variable's value, byte for byte, in a zeroizing
/// [`Held`] (no getter: it reaches the host only through the SDK's `resolve` answer).
///
/// # Errors
/// Fail-closed: an unset or non-UTF-8 variable (both "is unset") and an empty one are
/// refused, and so is a missing or blank `key`. A reference naming any setting but `key` is refused
/// naming it. The text names the variable, never its value.
pub fn resolve(
    settings: &serde_json::Map<String, serde_json::Value>,
) -> SecretResult<Held<Vec<u8>>> {
    let var = match settings.get(KEY).and_then(|v| v.as_str()) {
        Some(v) if !v.trim().is_empty() => v,
        _ => {
            return Err(SecretModuleError::invalid(
                "secret module 'env' requires settings.key naming the environment variable \
                 (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)",
            ))
        }
    };
    if let Some(other) = settings.keys().find(|k| k.as_str() != KEY) {
        return Err(SecretModuleError::invalid(format!(
            "secret module 'env' does not take settings.{other}; a reference names only \
             settings.key (fail-closed)"
        )));
    }
    // Like `std::env::var`, whose one `Err` covers a variable that is absent and one whose value is
    // not UTF-8, both answered "is unset"; an EMPTY value refused; any other value, a whitespace-only
    // one included, resolved untrimmed. The value is never quoted back. It is MOVED (never copied)
    // into a zeroizing holder before any judgement (`var` would drop a non-UTF-8 value unwiped
    // inside its error).
    let unset = || {
        SecretModuleError::not_found(format!(
            "secret env:{var} cannot resolve: environment variable '{var}' is unset"
        ))
    };
    let Some(value) = std::env::var_os(var) else {
        return Err(unset());
    };
    let bytes = value.into_encoded_bytes();
    let text = std::str::from_utf8(&bytes).is_ok();
    let held = Held::new(bytes);
    if !text {
        return Err(unset());
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

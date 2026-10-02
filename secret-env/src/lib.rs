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
//! It is an ordinary plugin (THE DESIGN §2): linked in the default build or dropped in, called
//! through the secret kind table by the one dispatcher, and loaded only when a reference names it.
//! Every refusal text is 1.5.5's, byte for byte; none carries the value.

#![forbid(unsafe_code)]

use busbar_contract::abi::secret::{
    ERROR_KIND_DENIED, ERROR_KIND_INTERNAL, ERROR_KIND_INVALID, ERROR_KIND_NOT_FOUND,
    ERROR_KIND_UNAVAILABLE,
};
use busbar_contract::secret::{SecretErrorKind, SecretModuleError, SecretResult};
use busbar_contract::secret_ref::SECRET_ENV_SETTING_KEY;

pub mod door;

/// The plugin's name (the manifest name).
pub const NAME: &str = "busbar-secret-env";

/// Resolve `settings` (`{ key: VAR }`) to the variable's value, byte for byte.
///
/// # Errors
/// Fail-closed: an unset, non-UTF-8, empty or whitespace-only variable is refused, and so is a
/// missing or blank `key`. The text names the variable, never its value.
pub fn resolve(settings: &serde_json::Map<String, serde_json::Value>) -> SecretResult<Vec<u8>> {
    let var = match settings
        .get(SECRET_ENV_SETTING_KEY)
        .and_then(|v| v.as_str())
    {
        Some(v) if !v.trim().is_empty() => v,
        _ => {
            return Err(SecretModuleError::invalid(
                "secret module 'env' requires settings.key naming the environment variable \
                 (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)",
            ))
        }
    };
    // `std::env::var` collapses two DIFFERENT operator errors into one `Err`: `NotPresent` (the
    // variable is absent) and `NotUnicode` (it is SET, and its value is not UTF-8). Reporting both
    // as "unset" is a wrong diagnosis rather than a missing one — an operator told their variable
    // is unset will set it again, which is the one action that cannot fix a value that is already
    // there and mis-encoded. `var_os` keeps the two apart: `None` is genuinely absent, `Some(raw)`
    // that fails `into_string` is present-but-unusable.
    match std::env::var_os(var) {
        None => Err(SecretModuleError::not_found(format!(
            "secret env:{var} cannot resolve: environment variable '{var}' is unset"
        ))),
        // The mis-encoded value is DROPPED, never quoted back: the whole point of this branch is
        // that the variable holds a credential, and an error message is not a place to put one.
        // Naming the variable is enough to act on.
        Some(raw) => match raw.into_string() {
            Err(_) => Err(SecretModuleError::invalid(format!(
                "secret env:{var} cannot resolve: environment variable '{var}' IS SET but its \
                 value is not valid UTF-8, so it cannot be read as a secret — this is an \
                 ENCODING problem, not a missing variable; re-export it as UTF-8 (setting it \
                 again will not help)"
            ))),
            Ok(v) if v.is_empty() => Err(SecretModuleError::invalid(format!(
                "secret env:{var} resolved to an EMPTY value; a secret must be non-empty \
                 (fail-closed)"
            ))),
            // A BLANK-but-present value is not a credential: "   " sent upstream as a bearer token
            // is the failure this refuses. Only an ENTIRELY-whitespace value is refused — the
            // accepted value is returned byte-for-byte, never trimmed, because trailing bytes are
            // part of the secret for a PEM chain and this is the RAW-bytes path.
            Ok(v) if v.trim().is_empty() => Err(SecretModuleError::invalid(format!(
                "secret env:{var} resolved to a BLANK value (whitespace only); a secret must \
                 carry actual content, and the variable IS SET — fix its value, not its \
                 presence (fail-closed)"
            ))),
            Ok(v) => Ok(v.into_bytes()),
        },
    }
}

/// A resolve's settings bytes as the JSON object they must be (empty = none).
///
/// # Errors
/// The bytes are not a JSON object.
pub fn settings_of(bytes: &[u8]) -> SecretResult<serde_json::Map<String, serde_json::Value>> {
    if bytes.is_empty() {
        return Ok(serde_json::Map::new());
    }
    serde_json::from_slice(bytes).map_err(|e| {
        SecretModuleError::invalid(format!("secret settings are not a JSON object: {e}"))
    })
}

/// The `abi::secret::ERROR_KIND_*` code of a [`SecretErrorKind`].
#[must_use]
pub const fn error_kind(kind: SecretErrorKind) -> u32 {
    match kind {
        SecretErrorKind::NotFound => ERROR_KIND_NOT_FOUND,
        SecretErrorKind::Unavailable => ERROR_KIND_UNAVAILABLE,
        SecretErrorKind::Denied => ERROR_KIND_DENIED,
        SecretErrorKind::Invalid => ERROR_KIND_INVALID,
        SecretErrorKind::Internal => ERROR_KIND_INTERNAL,
    }
}

#[cfg(test)]
#[path = "tests/lib_tests.rs"]
mod tests;

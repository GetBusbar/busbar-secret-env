// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! The `env` source's every fail-closed branch, with 1.5.5's texts (moved with the source from
//! busbar's `plugin-loader/src/builtin_secret.rs`).

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// A process-unique variable name, so parallel tests never share one.
fn unique(tag: &str) -> String {
    static CTR: AtomicU64 = AtomicU64::new(0);
    format!(
        "BUSBAR_SECRET_ENV_T_{tag}_{}_{}",
        std::process::id(),
        CTR.fetch_add(1, Ordering::SeqCst)
    )
}

fn key(v: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut m = serde_json::Map::new();
    m.insert("key".into(), v.into());
    m
}

#[test]
fn a_set_variable_resolves_byte_for_byte_untrimmed() {
    let var = unique("SET");
    std::env::set_var(&var, "  v\n");
    assert_eq!(resolve(&key(&var)).unwrap(), b"  v\n");
    std::env::remove_var(&var);
}

#[test]
fn an_unset_variable_is_not_found_naming_it() {
    let var = unique("UNSET");
    let e = resolve(&key(&var)).unwrap_err();
    assert_eq!(error_kind(e.kind), ERROR_KIND_NOT_FOUND);
    assert_eq!(
        e.message,
        format!("secret env:{var} cannot resolve: environment variable '{var}' is unset")
    );
}

#[test]
fn empty_and_blank_values_are_refused_without_the_value() {
    let (empty, blank) = (unique("EMPTY"), unique("BLANK"));
    std::env::set_var(&empty, "");
    std::env::set_var(&blank, " \t ");
    let e = resolve(&key(&empty)).unwrap_err();
    assert_eq!(
        e.message,
        format!("secret env:{empty} resolved to an EMPTY value; a secret must be non-empty (fail-closed)")
    );
    let e = resolve(&key(&blank)).unwrap_err();
    assert!(
        e.message.contains("BLANK value (whitespace only)"),
        "{}",
        e.message
    );
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    std::env::remove_var(&empty);
    std::env::remove_var(&blank);
}

#[cfg(unix)]
#[test]
fn a_mis_encoded_value_is_not_reported_as_unset_and_is_not_quoted() {
    use std::os::unix::ffi::OsStrExt;
    let var = unique("NOTUTF8");
    std::env::set_var(&var, std::ffi::OsStr::from_bytes(b"ab\xffcd"));
    let e = resolve(&key(&var)).unwrap_err();
    assert!(
        e.message
            .contains("IS SET but its value is not valid UTF-8"),
        "{}",
        e.message
    );
    assert!(
        !e.message.contains("is unset") && !e.message.contains("ab\u{fffd}cd"),
        "{}",
        e.message
    );
    std::env::remove_var(&var);
}

#[test]
fn a_missing_or_blank_key_is_invalid() {
    for s in [serde_json::Map::new(), key("  "), key("\t\n")] {
        let e = resolve(&s).unwrap_err();
        assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
        assert_eq!(
            e.message,
            "secret module 'env' requires settings.key naming the environment variable \
             (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)"
        );
    }
}

#[test]
fn settings_that_are_not_an_object_are_invalid() {
    assert!(settings_of(b"").unwrap().is_empty());
    let e = settings_of(b"[1]").unwrap_err();
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    assert!(
        e.message
            .starts_with("secret settings are not a JSON object: "),
        "{}",
        e.message
    );
}

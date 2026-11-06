// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! The `env` source's fail-closed branches and precise source diagnostics.

use super::*;
use busbar_contract::abi::sdk::life::Life;
use busbar_contract::abi::sdk::secret::error_kind;
use busbar_contract::abi::secret::{ERROR_KIND_INVALID, ERROR_KIND_NOT_FOUND};
use std::sync::atomic::{AtomicU64, Ordering};

/// The instance settings as the door judges them (`validate`, `open`, `refresh`).
fn instance_settings(bytes: &[u8]) -> Result<(), String> {
    crate::door::Env::validate(bytes).map_err(|refusal| refusal.text().unwrap().to_string())
}

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
    assert!(resolve(&key(&var)).expect("resolves") == Held::new(b"  v\n".to_vec()));
    std::env::remove_var(&var);
}

#[test]
fn an_unset_variable_is_not_found_naming_it() {
    let var = unique("UNSET");
    let e = resolve(&key(&var)).err().expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_NOT_FOUND);
    assert_eq!(
        e.message,
        format!("secret env:{var} cannot resolve: environment variable '{var}' is unset")
    );
}

/// A multi-byte UTF-8 value is valid text: it resolves byte for byte, not refused as mis-encoded.
#[test]
fn a_non_ascii_utf8_value_resolves_byte_for_byte() {
    let var = unique("NONASCII");
    std::env::set_var(&var, "p\u{e4}ssw\u{f6}rd-\u{1f511}");
    let resolved = resolve(&key(&var));
    std::env::remove_var(&var);
    assert!(
        resolved.expect("resolves")
            == Held::new("p\u{e4}ssw\u{f6}rd-\u{1f511}".as_bytes().to_vec())
    );
}

/// 1.5.5's `env` source (v1.5.5 crates/busbar/src/config/secret.rs `resolve_builtin`) refused only an
/// EMPTY value; a whitespace-only value resolved, byte for byte, and still does.
#[test]
fn an_empty_value_is_refused_and_a_blank_one_resolves_as_in_1_5_5() {
    let (empty, blank) = (unique("EMPTY"), unique("BLANK"));
    std::env::set_var(&empty, "");
    std::env::set_var(&blank, " \t ");
    let e = resolve(&key(&empty)).err().expect("refused");
    assert_eq!(
        e.message,
        format!("secret env:{empty} resolved to an EMPTY value; a secret must be non-empty (fail-closed)")
    );
    assert!(resolve(&key(&blank)).expect("resolves") == Held::new(b" \t ".to_vec()));
    std::env::remove_var(&empty);
    std::env::remove_var(&blank);
}

/// 1.6.0 hardening supersedes the legacy unset diagnostic: a present value with invalid
/// encoding is distinct from an absent variable, and its bytes must never enter the error.
#[cfg(unix)]
#[test]
fn a_mis_encoded_value_names_its_encoding_error_and_is_not_quoted() {
    use std::os::unix::ffi::OsStrExt;
    let var = unique("NOTUTF8");
    std::env::set_var(&var, std::ffi::OsStr::from_bytes(b"ab\xffcd"));
    let e = resolve(&key(&var)).err().expect("refused");
    assert_eq!(
        e.message,
        format!("secret env:{var} cannot resolve: environment variable '{var}' is not valid UTF-8")
    );
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    std::env::remove_var(&var);
}

#[test]
fn a_missing_or_blank_key_is_invalid() {
    for s in [serde_json::Map::new(), key("  "), key("\t\n")] {
        let e = resolve(&s).err().expect("refused");
        assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
        assert_eq!(
            e.message,
            "secret module 'env' requires settings.key naming the environment variable \
             (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)"
        );
    }
}

/// 1.5.5 read the key with `.and_then(|v| v.as_str())` (`resolve_builtin` / `SecretRef::env_var`),
/// so a `key` that is not a JSON string is refused with the same text as a missing one.
#[test]
fn a_non_string_key_is_invalid_as_in_1_5_5() {
    let mut obj = serde_json::Map::new();
    obj.insert("X".into(), "Y".into());
    for v in [
        serde_json::Value::from(5),
        serde_json::Value::from(true),
        serde_json::Value::Null,
        serde_json::Value::Array(vec!["A".into()]),
        serde_json::Value::Object(obj),
    ] {
        let mut s = serde_json::Map::new();
        s.insert("key".into(), v);
        let e = resolve(&s).err().expect("refused");
        assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
        assert_eq!(
            e.message,
            "secret module 'env' requires settings.key naming the environment variable \
             (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)"
        );
    }
}

/// 1.5.5 passed the key to `std::env::var` untrimmed: a padded key does not find the variable and
/// the "is unset" text names the padded key.
#[test]
fn a_key_is_not_trimmed_as_in_1_5_5() {
    let var = unique("PADDED");
    std::env::set_var(&var, "v");
    let padded = format!(" {var} ");
    let e = resolve(&key(&padded)).err().expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_NOT_FOUND);
    assert_eq!(
        e.message,
        format!("secret env:{padded} cannot resolve: environment variable '{padded}' is unset")
    );
    std::env::remove_var(&var);
}

/// A key that is not a valid variable name (a NUL byte or `=`) is refused INVALID, never looked up
/// (a lookup could match another variable's prefix), and never panics.
#[test]
fn a_key_with_nul_or_equals_is_invalid_not_a_panic() {
    for (k, shown) in [("A\u{0}B", "A\\0..."), ("A=B", "A=...")] {
        let e = resolve(&key(k)).err().expect("refused");
        assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
        assert_eq!(
            e.message,
            format!(
                "secret module 'env': settings.key '{shown}' is not a valid environment variable \
                 name (contains '=' or NUL; fail-closed)"
            )
        );
    }
}

#[test]
fn instance_lifecycle_rejects_non_objects_without_installing_state() {
    use crate::door::Env;
    use busbar_contract::abi::mechanism::call::Outcome;

    for bytes in [&b"[1]"[..], b"null", b"7", b"{"] {
        let validated = Env::validate(bytes).expect_err("validate refuses");
        let opened = Env::open(bytes, &[], 1).expect_err("open refuses");
        let refreshed = Env.refresh(bytes, &[], 2).expect_err("refresh refuses");
        for refusal in [validated, opened, refreshed] {
            assert_eq!(refusal.outcome(), Outcome::Failed);
            assert_eq!(refusal.text(), Some("settings: must be a JSON object"));
        }
    }
}

/// A reference names its variable and nothing else: any other setting (a typo, a key meant for
/// another source) is refused naming it, rather than ignored.
#[test]
fn a_reference_naming_another_setting_is_invalid_naming_it() {
    let var = unique("EXTRA");
    std::env::set_var(&var, "v");
    let mut s = key(&var);
    s.insert("path".into(), "/x".into());
    let e = resolve(&s).err().expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    assert_eq!(
        e.message,
        "secret module 'env' does not take settings.path; a reference names only settings.key \
         (fail-closed)"
    );
    std::env::remove_var(&var);
}

/// The source takes no instance settings: none or an empty object opens it, and a setting is
/// refused naming it, as are bytes that are not a JSON object.
#[test]
fn instance_settings_are_none_and_a_setting_is_refused_naming_it() {
    assert_eq!(instance_settings(b""), Ok(()));
    assert_eq!(instance_settings(b"{}"), Ok(()));
    assert_eq!(instance_settings(b" { } "), Ok(()));
    assert_eq!(
        instance_settings(br#"{"key":"X"}"#),
        Err(
            "secret module 'env' takes no settings; remove settings.key (each reference names \
             its variable in its own settings.key)"
                .to_string()
        )
    );
    for bad in [&b"[1]"[..], b"null", b"\"x\"", b"{"] {
        assert_eq!(
            instance_settings(bad),
            Err("settings: must be a JSON object".to_string()),
            "{}",
            String::from_utf8_lossy(bad)
        );
    }
}

#[test]
fn a_key_containing_equals_is_refused_not_resolved_as_another_variable() {
    let var = unique("EQ");
    std::env::set_var(&var, "user=x");
    let e = resolve(&key(&format!("{var}=user")))
        .err()
        .expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    assert!(!e.message.contains("x"), "{}", e.message);
    std::env::remove_var(&var);
}

#[test]
fn a_key_containing_nul_is_refused() {
    let e = resolve(&key("A\0B")).err().expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
}

#[test]
fn a_stray_setting_without_key_is_refused_naming_it() {
    let mut m = serde_json::Map::new();
    m.insert("name".into(), "X".into());
    let e = resolve(&m).err().expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    assert!(e.message.contains("settings.name"), "{}", e.message);
}

#[test]
fn an_extra_setting_precedes_missing_or_blank_key_but_plain_missing_keeps_its_words() {
    for (value, extra) in [
        (serde_json::json!({"kye":"X"}), "kye"),
        (serde_json::json!({"key":"  ","path":"/x"}), "path"),
    ] {
        let error = resolve(value.as_object().unwrap()).err().expect("refused");
        assert_eq!(error_kind(error.kind), ERROR_KIND_INVALID);
        assert_eq!(error.message, format!("secret module 'env' does not take settings.{extra}; a reference names only settings.key (fail-closed)"));
    }
    let empty = serde_json::Map::new();
    let error = resolve(&empty).err().expect("plain missing key refuses");
    assert_eq!(error.message, "secret module 'env' requires settings.key naming the environment variable (e.g. `{ env: MY_VAR }` or `{ module: env, settings: { key: MY_VAR } }`)");
}

/// A pasted `NAME=value` line in place of a name: the refusal names the variable part only, so the
/// value (the secret) never reaches the config error or the log.
#[test]
fn a_pasted_assignment_is_refused_without_echoing_its_value() {
    let e = resolve(&key("API_KEY=sk-live-0123456789"))
        .err()
        .expect("refused");
    assert_eq!(error_kind(e.kind), ERROR_KIND_INVALID);
    assert!(!e.message.contains("sk-live"), "{}", e.message);
    assert!(e.message.contains("'API_KEY=...'"), "{}", e.message);
}

/// A name carrying line breaks is escaped in every refusal, so the text stays one well-formed
/// reason the host accepts instead of a refusal "without saying why".
#[test]
fn names_with_line_breaks_are_escaped_in_every_refusal() {
    use busbar_contract::abi::mechanism::lifecycle::check_validate_refusal;
    let mut stray = serde_json::Map::new();
    stray.insert("x\n\ny".into(), "v".into());
    let unset = format!("{}\n\nZ", unique("NL"));
    for settings in [key("\nA=\n\nB"), stray, key(&unset)] {
        let e = resolve(&settings).err().expect("refused");
        assert!(!e.message.contains('\n'), "{:?}", e.message);
        assert_eq!(
            check_validate_refusal(e.message.as_bytes()),
            Ok(()),
            "{:?}",
            e.message
        );
    }
}

/// The door answers a cancelled resolve with the secret kind's ABORTED disposition.
#[test]
fn a_cancel_is_answered_aborted() {
    assert_eq!(
        <crate::door::Env as Life>::CANCEL,
        busbar_contract::abi::secret::cancel::ABORTED
    );
}

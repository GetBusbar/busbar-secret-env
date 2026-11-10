// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! Root source-resolution assertions transferred to the actual linked env owner.
//! Every resolution crosses SecretRows, the production dispatcher and the owner's SDK door.
use busbar_contract::{plugin::TestKernelSeal, redacted::Secret, secret::SecretAxis};
use busbar_plugin_loader::{
    dispatch::{DispatchConfig, Dispatcher},
    secret_calls::SecretRows,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, OnceLock},
};

fn dispatcher() -> Arc<Dispatcher> {
    static ONE: OnceLock<Arc<Dispatcher>> = OnceLock::new();
    ONE.get_or_init(|| {
        Arc::new(Dispatcher::new(
            DispatchConfig::default(),
            busbar_contract::services::test_caller_binding(),
        ))
    })
    .clone()
}
fn linked_axis() -> &'static SecretRows {
    static ONE: OnceLock<SecretRows> = OnceLock::new();
    ONE.get_or_init(|| {
        let mut rows = SecretRows::new(dispatcher, || None).with_disk_lane(|job| {
            job();
            Ok(())
        });
        rows.link(busbar_secret_env::door)
            .expect("the real owner door links");
        rows.declare_linked(busbar_secret_env::door, busbar_secret_env::DECLARES)
            .expect("the actual owner declaration reads");
        assert!(rows.linked("env"));
        rows
    })
}
fn reference(value: &str) -> serde_json::Map<String, serde_json::Value> {
    [("key".into(), value.into())].into_iter().collect()
}
fn resolve_linked(
    settings: &serde_json::Map<String, serde_json::Value>,
) -> Result<Secret<Vec<u8>>, String> {
    linked_axis()
        .shared("env")?
        .resolve_at_config(
            &serde_json::to_vec(settings).unwrap(),
            &busbar_contract::config_time::ConfigScope::config_time(),
        )
        .map_err(|error| error.text)
}
fn unique(tag: &str) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "BUSBAR_ROOT_LINKED_TRANSFER_{tag}_{}_{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    )
}
#[test]
fn resolve_linked_env_happy_path_reads_the_variable() {
    let var = unique("ENV_HAPPY");
    std::env::set_var(&var, "s3cr3t-value");
    let got = resolve_linked(&reference(&var)).unwrap();
    let got = got.expose(&TestKernelSeal).as_slice();
    assert_eq!(got, b"s3cr3t-value");
    std::env::remove_var(&var);
}

#[test]
fn resolve_linked_env_empty_value_is_fail_closed() {
    let var = unique("ENV_EMPTY");
    std::env::set_var(&var, "");
    let err = resolve_linked(&reference(&var)).unwrap_err();
    assert!(err.contains("EMPTY"), "{err}");
    assert!(err.contains(&var), "{err}");
    std::env::remove_var(&var);
}

#[test]
fn resolve_linked_env_unset_variable_is_an_error() {
    let var = unique("ENV_UNSET");
    std::env::remove_var(&var);
    let err = resolve_linked(&reference(&var)).unwrap_err();
    assert!(err.contains("unset"), "{err}");
    assert!(err.contains(&var), "{err}");
}

#[test]
fn resolve_linked_env_malformed_settings_is_a_shape_error_not_unknown_module() {
    let malformed = serde_json::Map::new();
    let err = resolve_linked(&malformed).unwrap_err();
    assert!(err.contains("settings.key"), "{err}");
    assert!(
        !err.contains("is not a built-in"),
        "a malformed env ref must not be misreported as an unknown module: {err}"
    );
}

#[test]
fn resolve_linked_env_blank_key_is_also_malformed() {
    let mut settings = serde_json::Map::new();
    settings.insert("key".to_string(), serde_json::Value::String("   ".into()));
    let blank = settings;
    let err = resolve_linked(&blank).unwrap_err();
    assert!(err.contains("settings.key"), "{err}");
}

#[test]
fn env_whitespace_only_name_is_rejected() {
    for bad in ["", "   ", "\t\n"] {
        let err = resolve_linked(&reference(bad)).expect_err(&format!(
            "whitespace-only env name {bad:?} must be rejected"
        ));
        assert!(
            err.contains("requires settings.key"),
            "whitespace-only env name {bad:?} must fail the settings-shape check, got: {err}"
        );
    }
}

#[test]
fn resolve_linked_env_whitespace_only_value_resolves_as_1_5_5_did() {
    for blank in ["   ", "\t", "\n\n", " \t\r\n "] {
        let var = unique("ENV_BLANK");
        std::env::set_var(&var, blank);
        let got = resolve_linked(&reference(&var)).unwrap_or_else(|e| {
            panic!("whitespace-only value {blank:?} resolves as in 1.5.5: {e}")
        });
        let got = got.expose(&TestKernelSeal).as_slice();
        assert_eq!(got, blank.as_bytes(), "its bytes, untrimmed");
        std::env::remove_var(&var);
    }
}

#[test]
fn resolve_linked_env_surrounding_whitespace_is_preserved_not_trimmed() {
    let var = unique("ENV_PADDED");
    std::env::set_var(&var, "  s3cr3t-value\n");
    let got = resolve_linked(&reference(&var)).unwrap();
    let got = got.expose(&TestKernelSeal).as_slice();
    assert_eq!(
        got, b"  s3cr3t-value\n",
        "a real credential resolves byte-for-byte; the blank guard must not trim it"
    );
    std::env::remove_var(&var);
}

#[cfg(unix)]
#[test]
fn resolve_linked_env_mis_encoded_value_refuses_in_1_5_5s_words() {
    use std::os::unix::ffi::OsStrExt;
    let var = unique("ENV_NOT_UNICODE");

    std::env::set_var(&var, std::ffi::OsStr::from_bytes(&[0x73, 0xff, 0x74]));
    let err = resolve_linked(&reference(&var))
        .expect_err("a mis-encoded variable cannot resolve to a usable secret");
    assert_eq!(
        err,
        format!("secret env:{var} cannot resolve: environment variable '{var}' is not valid UTF-8"),
        "1.6.0 distinguishes a present, mis-encoded value from an unset variable"
    );
    std::env::remove_var(&var);
}

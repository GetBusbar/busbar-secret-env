// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

use crate::distribution::config_validation_application::{
    make_provider, make_root_cfg, proto_a, source_fill, source_path, valid_maps, validate,
};
use busbar_kernel::config;
use std::collections::HashMap;

#[test]
fn test_validate_operator_token_secret_module_checked() {
    let (providers, models, pools) = valid_maps();
    let mut cfg = make_root_cfg(providers, models, pools);
    let mut auth = config::AuthCfg::default_none();
    let mut entry = config::AuthChainEntry::bare(config::operator_provider());
    entry.token = Some(without_scalar(source_fill("fixture-input")));
    auth.admin_auth = vec![entry];
    cfg.auth = Some(auth);
    let errs = validate(&cfg).expect_err("an env secret ref without settings.key must fail");

    let path = format!("auth.admin_auth.{}.token", config::operator_provider());
    assert!(
        errs.iter()
            .any(|e| e.contains(&path) && e.contains("requires settings.key")),
        "expected the env-shape error; got: {errs:?}"
    );
}

#[test]
fn test_validate_secret_module_resolvability() {
    let (mut providers, models, pools) = valid_maps();
    providers.get_mut("myprovider").unwrap().api_key =
        config::SecretRef::Source(busbar_contract::secret_ref::SourceReference {
            module: "fixture-source".into(),
            settings: Default::default(),
        });
    let cfg = make_root_cfg(providers, models, pools);
    assert!(
        validate(&cfg).is_ok(),
        "a plugin-backed api_key module must not be rejected by pre-registry validate; got: {:?}",
        validate(&cfg)
    );

    let (providers, models, pools) = valid_maps();
    let mut cfg = make_root_cfg(providers, models, pools);
    cfg.tls = Some(config::TlsCfg {
        cert: config::SecretRef::Source(busbar_contract::secret_ref::SourceReference {
            module: "fixture-source".into(),
            settings: Default::default(),
        }),
        key: without_scalar(source_path("fixture-input")),
        client_ca: Some(without_scalar(source_fill("fixture-input"))),
    });
    let errs = validate(&cfg).expect_err("structurally-bad tls secret refs must fail validation");

    assert!(
        !errs.iter().any(|e| e.contains("tls.cert")),
        "a plugin-backed tls.cert must not be rejected by pre-registry validate; got: {errs:?}"
    );
    assert!(
        errs.iter()
            .any(|e| e.contains("tls.key") && e.contains("requires settings.path")),
        "tls.key file ref without settings.path must error; got: {errs:?}"
    );
    assert!(
        errs.iter()
            .any(|e| e.contains("tls.client_ca") && e.contains("requires settings.key")),
        "tls.client_ca env ref without settings.key must error; got: {errs:?}"
    );

    let (providers, models, pools) = valid_maps();
    let mut cfg = make_root_cfg(providers, models, pools);
    cfg.tls = Some(config::TlsCfg {
        cert: crate::distribution::config_validation_application::source_path(
            "/run/secrets/tls-cert.pem",
        ),
        key: crate::distribution::config_validation_application::source_path(
            "/run/secrets/tls-key.pem",
        ),
        client_ca: Some(
            crate::distribution::config_validation_application::source_fill("BUSBAR_CLIENT_CA_PEM"),
        ),
    });
    assert!(
        validate(&cfg).is_ok(),
        "well-formed tls secret refs must validate; got: {:?}",
        validate(&cfg)
    );

    let (providers, models, pools) = valid_maps();
    let mut cfg = make_root_cfg(providers, models, pools);
    let mut auth = config::AuthCfg::default_none();
    auth.signing_key = Some(config::SecretRef::Source(
        busbar_contract::secret_ref::SourceReference {
            module: "fixture-source".into(),
            settings: Default::default(),
        },
    ));
    cfg.auth = Some(auth);
    assert!(
        validate(&cfg).is_ok(),
        "a plugin-backed signing_key module must not be rejected by pre-registry validate; got: {:?}",
        validate(&cfg)
    );
}

#[test]
fn test_validate_rejects_empty_canonical_builtin_secret_ref() {
    let empty_env: config::SecretRef =
        serde_yaml::from_str("{ module: env, settings: { key: \"\" } }")
            .expect("canonical empty-key env ref deserializes (the guard is sugar-only)");
    assert_eq!(
        empty_env,
        source_fill(""),
        "precondition: the canonical reference retains its empty scalar"
    );

    let mut providers = HashMap::new();
    let mut p = make_provider(proto_a(), "https://api.acme.example.com", "IGNORED");
    p.api_key = empty_env;
    providers.insert("acme".to_string(), p);
    let errs = validate(&make_root_cfg(providers, HashMap::new(), HashMap::new()))
        .expect_err("an empty canonical env secret key must fail validation");
    assert!(
        errs.iter().any(|e| e.contains("providers.acme.api_key")
            && e.contains("'env'")
            && e.to_lowercase().contains("non-empty")),
        "expected a non-empty settings.key error naming providers.acme.api_key; got: {errs:?}"
    );

    let good: config::SecretRef =
        serde_yaml::from_str("{ module: env, settings: { key: REAL_KEY } }")
            .expect("canonical non-empty env ref deserializes");
    let mut providers = HashMap::new();
    let mut p = make_provider(proto_a(), "https://api.acme.example.com", "IGNORED");
    p.api_key = good;
    providers.insert("acme".to_string(), p);
    let errs = validate(&make_root_cfg(providers, HashMap::new(), HashMap::new()))
        .err()
        .unwrap_or_default();
    assert!(
        !errs
            .iter()
            .any(|e| e.contains("providers.acme.api_key") && e.to_lowercase().contains("non-empty")),
        "a non-empty canonical key must not raise the empty-secret error; got: {errs:?}"
    );
}

fn without_scalar(reference: config::SecretRef) -> config::SecretRef {
    let config::SecretRef::Source(mut source) = reference else {
        panic!("fixture source reference")
    };
    source.settings = Default::default();
    config::SecretRef::Source(source)
}

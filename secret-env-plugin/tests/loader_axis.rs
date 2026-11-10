// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! Actual environment-source behavior transferred from the core loader's generic witness tests.

use busbar_contract::abi::secret::ERROR_KIND_NOT_FOUND;
use busbar_contract::config_time::ConfigScope;
use busbar_contract::plugin::TestKernelSeal;
use busbar_contract::secret::{SecretAxis, SecretCalls};
use busbar_plugin_loader::boot::{Candidate, Origin};
use busbar_plugin_loader::dispatch::{rendering_of, DispatchConfig, Dispatcher};
use busbar_plugin_loader::secret_calls::SecretRows;
use std::sync::Arc;

const HIT: &str = "BUSBAR_OWNER_LOADER_AXIS_HIT";
const MISS: &str = "BUSBAR_OWNER_LOADER_AXIS_MISS";
const MATERIAL: &str = "owner-axis-secret-material";

fn dispatcher() -> Arc<Dispatcher> {
    static ONE: std::sync::OnceLock<Arc<Dispatcher>> = std::sync::OnceLock::new();
    ONE.get_or_init(|| {
        let dispatcher = Dispatcher::new(
            DispatchConfig::default(),
            busbar_contract::services::test_caller_binding(),
        );
        assert!(dispatcher.lend_seal(TestKernelSeal));
        Arc::new(dispatcher)
    })
    .clone()
}

fn transcript(opened: &dyn SecretCalls) -> Vec<String> {
    [HIT, MISS]
        .into_iter()
        .map(|key| {
            let settings = serde_json::to_vec(&serde_json::json!({"key": key})).unwrap();
            match opened.resolve(&settings) {
                Ok(value) => format!(
                    "ok {}",
                    String::from_utf8_lossy(value.expose(&TestKernelSeal))
                ),
                Err(error) => format!("refused {} {}", error.error_kind, error.text),
            }
        })
        .collect()
}

#[test]
fn actual_environment_resolution_agrees_across_linked_shared_and_dropped_axes() {
    std::env::set_var(HIT, MATERIAL);
    std::env::remove_var(MISS);
    let mut linked = SecretRows::new(dispatcher, || None);
    linked.link(busbar_secret_env::door).unwrap();
    linked
        .declare_linked(busbar_secret_env::door, busbar_secret_env::DECLARES)
        .unwrap();
    let image = std::path::PathBuf::from(
        std::env::var_os("BUSBAR_PLUGIN_CDYLIB")
            .expect("build the actual plugin and name its cdylib"),
    );
    assert!(image.is_absolute() && image.is_file());
    let candidate = Candidate::from_rendering(
        rendering_of(busbar_secret_env::door).unwrap(),
        None,
        Origin::Dropped {
            file: "busbar-secret-env.tar.gz".into(),
            bytes: Arc::new(std::fs::read(image).unwrap()),
        },
    )
    .unwrap();
    let dropped = SecretRows::new(dispatcher, || None);
    dropped.set_dropped([candidate]);
    assert!(linked.answers(busbar_secret_env::NAME) && linked.linked(busbar_secret_env::NAME));
    assert!(dropped.answers(busbar_secret_env::NAME) && !dropped.linked(busbar_secret_env::NAME));
    assert!(dropped.shared(busbar_secret_env::NAME).is_err());
    let shared = linked.shared(busbar_secret_env::NAME).unwrap();
    let expected = transcript(shared.as_ref());
    assert_eq!(expected[0], format!("ok {MATERIAL}"));
    assert!(expected[1].starts_with(&format!("refused {ERROR_KIND_NOT_FOUND} ")));
    for axis in [&linked, &dropped] {
        let settings = serde_json::from_value(serde_json::json!({})).unwrap();
        let opened = axis
            .open(busbar_secret_env::NAME, Some(&settings), &|_| {
                panic!("no referenced settings")
            })
            .unwrap();
        assert_eq!(transcript(opened.as_ref()), expected);
        let reference = serde_json::from_value(serde_json::json!({"key": HIT})).unwrap();
        assert_eq!(axis.check(busbar_secret_env::NAME, &reference), Ok(()));
        let settings = serde_json::to_vec(&serde_json::json!({"key": HIT})).unwrap();
        assert_eq!(
            opened
                .resolve_at_config(&settings, &ConfigScope::config_time())
                .unwrap()
                .expose(&TestKernelSeal),
            MATERIAL.as_bytes()
        );
    }
    std::env::remove_var(HIT);
}

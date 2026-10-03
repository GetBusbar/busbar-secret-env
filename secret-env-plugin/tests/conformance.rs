// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! **THE PUBLISHED CONFORMANCE SUITE, RUN BY THIS PLUGIN** (busbar TODO ABI-b4; OWNER 2026-10-03:
//! plugins test themselves against busbar). busbar's suite, at the commit this repo pins
//! (`.busbar-ref`), drives the `env` source two ways through the one loader: LINKED (the logic
//! crate's `door::door`) and DROPPED IN (this crate's built cdylib), over the secret kind's script
//! with the inputs in `conformance.json`; every step's crossings exactly at the script's pin, the
//! two folds equal, and the suite's RED arms kept. `plugin-ci.yml` runs it under `--release`.

busbar_plugin_loader::conformance_suite! {
    door: busbar_secret_env::door::door,
    cdylib: "busbar_secret_env_plugin",
    inputs: include_str!("conformance.json"),
}

// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! THE DECLARES DOCUMENT, as the host reads it: the `env` source reads the process environment
//! only, so it declares nothing: no disk need (it never runs on the disk lane), no network need,
//! no diagnostics. Read through the loader's ONE reader of a declares document, at the busbar
//! commit this repo pins.

use busbar_plugin_loader::sign::Declares;

#[test]
fn the_declares_document_declares_nothing() {
    let declares = Declares::from_declares_json(busbar_secret_env::DECLARES.as_bytes())
        .expect("the declares document reads");
    assert_eq!(
        declares,
        Declares::default(),
        "the env source needs no disk, no network and raises no diagnostic"
    );
}

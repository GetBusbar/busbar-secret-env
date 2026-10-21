// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! THE SHORTHAND THIS SOURCE ANSWERS TO, as the host reads it (busbar audit contract-admin #10,
//! Law 0/1: core names kinds, never instances): the `{ env: X }` shorthand is this plugin's own
//! declaration, so its Statement states the word, the settings key the value lands under and what
//! a refusal says the value names. The kernel reads these off the loaded Statements and spells
//! none of them. Read through the loader's linked row, at the busbar commit this repo pins.

use busbar_contract::abi::mechanism::door::{REWRITE_SUGAR, REWRITE_SUGAR_NAMES};
use busbar_contract::abi::mechanism::rendering::read;
use busbar_plugin_loader::dispatch::LinkedRow;

/// The rewrites the linked door's Statement states, `(class, from, to)`.
fn rewrites() -> Vec<(u32, String, String)> {
    let row =
        LinkedRow::of(busbar_secret_env::door::door).expect("the linked door states its Statement");
    read(&row.statement)
        .expect("the Statement rendering reads back")
        .rewrites
}

#[test]
fn the_statement_declares_its_shorthand_and_the_settings_key_it_lands_under() {
    let r = rewrites();
    assert!(
        r.contains(&(REWRITE_SUGAR, "env".to_string(), "key".to_string())),
        "the `{{ env: X }}` shorthand must state its settings key `key`: {r:?}"
    );
}

#[test]
fn the_statement_declares_what_the_shorthand_value_names() {
    let r = rewrites();
    assert!(
        r.contains(&(
            REWRITE_SUGAR_NAMES,
            "env".to_string(),
            "the environment variable name".to_string()
        )),
        "the `{{ env: X }}` shorthand must state what its value names: {r:?}"
    );
}

// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! The `rewrite` helper the Statement's rewrite rows are built with: every row it builds names this
//! source by its word, keeps its class, zeroes the padding and carries `to` as given.

use busbar_contract::abi::mechanism::call::AbiStr;
use busbar_contract::abi::mechanism::check::check_rewrites;
use busbar_contract::abi::mechanism::door::{
    REWRITE_ALIAS, REWRITE_SUGAR, REWRITE_SUGAR_NAMES, REWRITE_SUGAR_ROLE, REWRITE_SUGAR_SHOWN,
};

/// `s` is present and spells exactly `text`.
#[allow(unsafe_code)]
fn names(s: AbiStr, text: &str) -> bool {
    // SAFETY: every `AbiStr` here was built by `rewrite` from a `'static` str.
    !s.is_absent() && unsafe { s.bytes() } == text.as_bytes()
}

#[test]
fn rewrite_names_the_env_word_and_carries_class_and_target() {
    for (class, to) in [
        (REWRITE_ALIAS, ""),
        (REWRITE_SUGAR, "some-key"),
        (REWRITE_SUGAR_SHOWN, "PLACEHOLDER"),
        (REWRITE_SUGAR_NAMES, "what the value names"),
        (REWRITE_SUGAR_ROLE, "fill"),
    ] {
        let r = super::rewrite(class, to);
        assert_eq!(r.class, class);
        assert_eq!(r._reserved, 0, "padding is zero");
        assert!(
            names(r.from, crate::WORD),
            "class {class}: `from` must be the source's word {:?}",
            crate::WORD
        );
        assert!(names(r.to, to), "class {class}: `to` must be {to:?} itself");
        check_rewrites(&[r]).unwrap_or_else(|e| {
            panic!("class {class}: the built rewrite breaks the contract's rule: {e:?}")
        });
    }
}

#[test]
fn the_stated_rewrites_satisfy_the_contracts_rewrite_rules() {
    check_rewrites(super::REWRITES).expect("the Statement's rewrite table passes check_rewrites");
}

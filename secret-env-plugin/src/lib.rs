// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! The `env` secret source as a droppable `kind: secret` plugin: the logic crate re-exported whole,
//! and its door exported as this image's one symbol, `busbar_plugin_door`.

#![deny(unsafe_code)]

pub use busbar_secret_env::*;

/// The exported door, behind `dropped-in` (the cdylib build only): the macro's `#[no_mangle]` symbol is
/// the one `unsafe` exemption.
#[cfg(feature = "dropped-in")]
#[allow(unsafe_code)]
mod exported {
    busbar_contract::export_door!(busbar_secret_env::door::door);
}

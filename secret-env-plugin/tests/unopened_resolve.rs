// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! The linked door's `resolve` slot, called raw with NO instance (a host that skipped `open`, or
//! one the loader's own guard does not stop): the plugin itself refuses, and no material reaches
//! the host's buffer even though the named variable is set.

use busbar_contract::abi::mechanism::call::{Blob, Outcome, BLOB_JSON};
use busbar_contract::abi::secret::{self, Ops, ResolveIn, ResolveOut};
use busbar_plugin_loader::dispatch::{in_head, out_head, NO_BLOB};

const SET: &str = "BUSBAR_SECRET_ENV_UNOPENED_SET";
const VALUE: &str = "unopened-must-not-leak";

/// Call the linked door's `resolve` with `instance`, the reference `{"key": SET}` and a buffer of
/// `VALUE.len()` bytes; answer the outcome, the out struct and the buffer afterwards.
fn resolve_raw(instance: *mut std::ffi::c_void) -> (Outcome, ResolveOut, Vec<u8>) {
    let ops = busbar_secret_env::door::door().ops().cast::<Ops>();
    // SAFETY: the secret kind's door answers a table that leads with `OpsHead` and is a whole
    // `secret::Ops` (its kind), alive for the program.
    let resolve = unsafe { (*ops).resolve }.expect("the env door wires resolve");
    let settings = format!(r#"{{"key":"{SET}"}}"#).into_bytes();
    let mut buffer = vec![0u8; VALUE.len()];
    let mut head = in_head();
    head.size = size_of::<ResolveIn>() as u32;
    head.op = secret::slot::RESOLVE;
    let input = ResolveIn {
        head,
        settings: Blob {
            ptr: settings.as_ptr(),
            len: settings.len(),
            fmt: BLOB_JSON,
            flags: 0,
        },
        secret_buf: buffer.as_mut_ptr(),
        secret_cap: buffer.len(),
    };
    let mut out_h = out_head();
    out_h.size = size_of::<ResolveOut>() as u32;
    let mut out = ResolveOut {
        head: out_h,
        secret: NO_BLOB,
        error_kind: 0,
        _reserved: 0,
        needed: 0,
    };
    // SAFETY: `input` and `out` are this slot's whole `in`/`out`, live for the call; `settings` and
    // `buffer` outlive it.
    let raw = unsafe {
        resolve(
            instance,
            std::ptr::from_ref(&input).cast(),
            std::ptr::from_mut(&mut out).cast(),
        )
    };
    (raw.outcome(), out, buffer)
}

#[test]
fn resolve_without_an_opened_instance_is_refused_and_writes_no_material() {
    // Edition 2021, and this test binary's only test: no other thread reads the environment.
    std::env::set_var(SET, VALUE);
    let (outcome, out, buffer) = resolve_raw(std::ptr::null_mut());
    assert_eq!(
        outcome,
        Outcome::Refused,
        "a resolve with no instance must be refused by the plugin, not served"
    );
    assert_eq!(out.head.outcome.outcome(), Outcome::Refused);
    assert!(out.secret.ptr.is_null(), "no material blob is answered");
    assert_eq!(out.secret.len, 0);
    assert_eq!(out.error_kind, 0, "a refusal is not a FAILED resolve");
    assert_eq!(out.needed, 0);
    assert!(
        buffer.iter().all(|b| *b == 0),
        "the set variable's value must not reach the host buffer"
    );
}

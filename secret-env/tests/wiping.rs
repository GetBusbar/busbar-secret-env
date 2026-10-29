// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! Every heap block the source frees while it resolves a variable must have been wiped first: a
//! value it refuses (one that is not UTF-8) is read into the process's heap and must not be left
//! there when it drops. This test binary installs a global allocator whose `dealloc` scans the
//! block being freed for a canary while armed. It is its own binary so the crate's
//! `forbid(unsafe_code)` does not apply to the allocator wrapper.
//!
//! Not covered, and not claimable: the process environment itself (`environ`) holds the value for
//! the life of the process; it is the operator's, not this source's.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};

const CANARY: &[u8] = b"CANARY-ENV-VALUE-5d1e";

static ARMED: AtomicBool = AtomicBool::new(false);
static SAW: AtomicBool = AtomicBool::new(false);

/// One armed test at a time: `ARMED` and `SAW` are the process's.
static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A naive byte search; allocates nothing.
fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.len() >= needle.len() && hay.windows(needle.len()).any(|w| w == needle)
}

struct Scanning;

// SAFETY: every call forwards unchanged to `System`; `dealloc` only reads the block it is about to
// free (and `realloc` is the trait's default, so a grown block's old storage also reaches `dealloc`).
unsafe impl GlobalAlloc for Scanning {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ARMED.load(Ordering::SeqCst) {
            let block = std::slice::from_raw_parts(ptr, layout.size());
            if contains(block, CANARY) {
                SAW.store(true, Ordering::SeqCst);
            }
        }
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static ALLOCATOR: Scanning = Scanning;

/// RED before the sibling review: `std::env::var` dropped a non-UTF-8 value inside its error
/// (`VarError::NotUnicode`) unwiped.
#[cfg(unix)]
#[test]
fn a_refused_non_utf8_value_is_wiped_before_it_is_freed() {
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    use std::os::unix::ffi::OsStrExt;
    let var = "BUSBAR_SECRET_ENV_WIPING_NOT_UTF8";
    let mut value = CANARY.to_vec();
    value.push(0xff);
    // Set before arming: `set_var`'s own copies are the test's, not the source's.
    std::env::set_var(var, std::ffi::OsStr::from_bytes(&value));
    let mut settings = serde_json::Map::new();
    settings.insert("key".into(), var.into());

    ARMED.store(true, Ordering::SeqCst);
    let refused = busbar_secret_env::resolve(&settings);
    drop(refused);
    ARMED.store(false, Ordering::SeqCst);

    assert!(
        !SAW.load(Ordering::SeqCst),
        "a refused value reached the allocator unwiped"
    );
    std::env::remove_var(var);
}

/// The credential chip, row 6 (host-owned secret bytes): a RESOLVED value is held only in a
/// zeroizing holder, so when the source's answer drops (the SDK has written it into the host's
/// buffer), its heap block is wiped before it is freed. RED while `resolve` answered a plain
/// `Vec<u8>`.
#[test]
fn a_resolved_value_is_wiped_before_it_is_freed() {
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let var = "BUSBAR_SECRET_ENV_WIPING_RESOLVED";
    std::env::set_var(var, std::str::from_utf8(CANARY).expect("ascii"));
    let mut settings = serde_json::Map::new();
    settings.insert("key".into(), var.into());

    ARMED.store(true, Ordering::SeqCst);
    let resolved = busbar_secret_env::resolve(&settings);
    assert!(resolved.is_ok(), "the variable resolves");
    drop(resolved);
    ARMED.store(false, Ordering::SeqCst);

    assert!(
        !SAW.load(Ordering::SeqCst),
        "a resolved value reached the allocator unwiped"
    );
    std::env::remove_var(var);
}

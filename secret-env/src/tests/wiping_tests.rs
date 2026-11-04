// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Busbar Inc and contributors

//! Observe the source's owned value, not the process environment or arbitrary heap storage.
//! An unregistered copy is out of scope: finding one would read memory the probe cannot prove is
//! initialized.

#[allow(unsafe_code)]
#[path = "wipe_probe.rs"]
mod probe;

use busbar_contract::abi::sdk::cred::Held;

const VALUE: &[u8] = b"CANARY-ENV-VALUE-5d1e";

pub(super) fn watch(bytes: &[u8]) {
    probe::watch("environment value", bytes);
}

#[test]
fn a_resolved_value_is_wiped_before_it_is_freed() {
    let var = format!("BUSBAR_SECRET_ENV_WIPING_RESOLVED_{}", std::process::id());
    std::env::set_var(&var, std::str::from_utf8(VALUE).unwrap());
    let settings = serde_json::json!({ "key": var })
        .as_object()
        .unwrap()
        .clone();

    let observed = probe::Probe::start();
    let resolved = super::resolve(&settings).expect("the variable resolves");
    assert!(
        resolved == Held::new(VALUE.to_vec()),
        "the actual value resolves"
    );
    drop(resolved);
    let report = observed.finish();

    std::env::remove_var(&var);
    report.assert_wiped(&["environment value"]);
}

#[cfg(unix)]
#[test]
fn a_refused_non_utf8_value_is_wiped_before_it_is_freed() {
    use busbar_contract::secret::SecretErrorKind;
    use std::os::unix::ffi::OsStrExt;

    let var = format!("BUSBAR_SECRET_ENV_WIPING_NOT_UTF8_{}", std::process::id());
    let mut value = VALUE.to_vec();
    value.push(0xff);
    std::env::set_var(&var, std::ffi::OsStr::from_bytes(&value));
    let settings = serde_json::json!({ "key": var })
        .as_object()
        .unwrap()
        .clone();

    let observed = probe::Probe::start();
    let refused = super::resolve(&settings)
        .err()
        .expect("non-UTF-8 is refused");
    assert_eq!(refused.kind, SecretErrorKind::Invalid);
    assert!(!refused
        .message
        .contains(std::str::from_utf8(VALUE).unwrap()));
    drop(refused);
    let report = observed.finish();

    std::env::remove_var(&var);
    report.assert_wiped(&["environment value"]);
}

#[allow(unsafe_code)]
fn resolve_through_the_actual_door(
    settings: &[u8],
    buffer: &mut [u8],
) -> (busbar_contract::abi::mechanism::call::Outcome, usize) {
    use busbar_contract::abi::mechanism::call::{Blob, InHead, OutHead, Outcome, BLOB_JSON};
    use busbar_contract::abi::mechanism::lifecycle::{slot, OpenIn, OpenOut};
    use busbar_contract::abi::secret::{self, Ops, ResolveIn, ResolveOut};
    use std::mem::size_of;

    // SAFETY: the plugin's own macro door states its static secret Ops table.
    let ops = unsafe { &*crate::door::door().ops().cast::<Ops>() };
    // SAFETY: these ABI records contain integers, raw pointers and optional function pointers;
    // their all-zero representation is valid. Sizes/slots and lent buffers are set before calls.
    let mut input: OpenIn = unsafe { std::mem::zeroed() };
    // SAFETY: OpenOut has the same all-zero-valid field types; it is host-owned output storage.
    let mut opened: OpenOut = unsafe { std::mem::zeroed() };
    input.head.size = size_of::<OpenIn>() as u32;
    input.head.op = slot::OPEN;
    input.generation = 1;
    opened.head.size = size_of::<OpenOut>() as u32;
    // SAFETY: this is the crate's live SDK lifecycle slot. The initialized, correctly sized
    // OpenIn and exclusive OpenOut remain aligned and alive through this synchronous call;
    // the zeroed absent settings/secrets/host spans contain no borrowed payload to retain.
    let outcome = unsafe {
        (ops.head.open.unwrap())(
            std::ptr::null_mut(),
            (&input as *const OpenIn).cast(),
            (&mut opened as *mut OpenOut).cast(),
        )
    }
    .outcome();
    assert_eq!(outcome, Outcome::Ready);
    assert!(!opened.instance.is_null());

    // SAFETY: ResolveIn/ResolveOut are all-zero-valid ABI records, fully sized and lent below.
    let mut input: ResolveIn = unsafe { std::mem::zeroed() };
    // SAFETY: ResolveOut contains only all-zero-valid ABI fields and is writable host storage.
    let mut output: ResolveOut = unsafe { std::mem::zeroed() };
    input.head.size = size_of::<ResolveIn>() as u32;
    input.head.op = secret::slot::RESOLVE;
    output.head.size = size_of::<ResolveOut>() as u32;
    input.settings = Blob {
        ptr: settings.as_ptr(),
        len: settings.len(),
        fmt: BLOB_JSON,
        flags: 0,
    };
    input.secret_buf = buffer.as_mut_ptr();
    input.secret_cap = buffer.len();
    let observed = probe::Probe::start();
    // SAFETY: the same door opened this live instance above. Input/output are initialized,
    // correctly sized and aligned; settings and the exclusive writable secret buffer remain
    // alive for the call. Env::Resolve is synchronous and retains none of those frame loans.
    let outcome = unsafe {
        (ops.resolve.unwrap())(
            opened.instance,
            (&input as *const ResolveIn).cast(),
            (&mut output as *mut ResolveOut).cast(),
        )
    }
    .outcome();
    let report = observed.finish();

    // SAFETY: InHead and OutHead also admit all-zero; close names the exact opened instance,
    // with the right slot/size, after the synchronous resolve returned and released its loans.
    let mut close: InHead = unsafe { std::mem::zeroed() };
    // SAFETY: OutHead is all-zero-valid writable ABI storage.
    let mut closed: OutHead = unsafe { std::mem::zeroed() };
    close.size = size_of::<InHead>() as u32;
    close.op = slot::CLOSE;
    closed.size = size_of::<OutHead>() as u32;
    // SAFETY: close consumes the instance opened by this door exactly once, after resolve
    // returned with no pending work. Its initialized, sized heads live through the call and
    // the instance is never used again after close returns.
    let closed = unsafe {
        (ops.head.close.unwrap())(
            opened.instance,
            (&close as *const InHead).cast(),
            (&mut closed as *mut OutHead).cast(),
        )
    }
    .outcome();
    assert_eq!(closed, Outcome::Ready);
    report.assert_wiped(&["environment value"]);
    assert_eq!(output.head.lease, 0);
    if outcome == Outcome::Ready {
        assert_eq!(output.secret.ptr, buffer.as_ptr());
        assert_eq!(output.secret.len, VALUE.len());
    } else {
        assert!(output.secret.ptr.is_null());
        assert_eq!(output.secret.len, 0);
    }
    (outcome, output.needed as usize)
}

#[test]
fn the_actual_door_wipes_the_resolved_value_on_success_and_short_buffer() {
    use busbar_contract::abi::mechanism::call::Outcome;

    let var = format!("BUSBAR_SECRET_ENV_WIPING_DOOR_{}", std::process::id());
    std::env::set_var(&var, std::str::from_utf8(VALUE).unwrap());
    let settings = serde_json::to_vec(&serde_json::json!({"key": var})).unwrap();
    let mut full = [0_u8; VALUE.len()];
    let full_answer = resolve_through_the_actual_door(&settings, &mut full);
    let mut short = [0_u8; VALUE.len() - 1];
    let short_answer = resolve_through_the_actual_door(&settings, &mut short);
    std::env::remove_var(&var);

    assert_eq!(full_answer, (Outcome::Ready, 0));
    assert_eq!(full.as_slice(), VALUE);
    assert_eq!(short_answer, (Outcome::Failed, VALUE.len()));
    assert!(short.iter().all(|byte| *byte == 0));
    full.fill(0);
}

// pattern: Imperative Shell

//! Small raw-WASM ABI. JavaScript owns the wrapper; this module only translates
//! bounded memory and status operations into the existing Rust bridge.

#![allow(unsafe_code)]

use super::{Encoder, EncoderError, EncoderStatus};
use std::{
    alloc::{Layout, alloc, dealloc},
    cell::RefCell,
    panic::{AssertUnwindSafe, catch_unwind},
    slice,
};

const STATUS_NEED_MORE_INPUT: i32 = 0;
const STATUS_BITSTREAM_AVAILABLE: i32 = 1;
const STATUS_END_OF_STREAM: i32 = 3;
const STATUS_ERROR: i32 = -1;
const MAX_WASM_ALLOCATION_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy)]
struct WasmAllocation {
    pointer: u32,
    length: usize,
}

thread_local! {
    static ENCODERS: RefCell<Vec<Option<Encoder>>> = const { RefCell::new(Vec::new()) };
    static ALLOCATIONS: RefCell<Vec<WasmAllocation>> = const { RefCell::new(Vec::new()) };
}

fn with_encoder<R>(handle: u32, operation: impl FnOnce(&mut Encoder) -> R) -> Option<R> {
    let index = usize::try_from(handle).ok()?.checked_sub(1)?;
    ENCODERS.with(|encoders| {
        encoders
            .borrow_mut()
            .get_mut(index)
            .and_then(Option::as_mut)
            .map(operation)
    })
}

fn guarded_status(handle: u32, operation: impl FnOnce(&mut Encoder) -> EncoderStatus) -> i32 {
    match catch_unwind(AssertUnwindSafe(|| with_encoder(handle, operation))) {
        Ok(Some(status)) => with_encoder(handle, |encoder| {
            if encoder.last_error().is_some() {
                STATUS_ERROR
            } else {
                status_code(status)
            }
        })
        .unwrap_or(STATUS_ERROR),
        Ok(None) => STATUS_ERROR,
        Err(_) => {
            let _ = with_encoder(handle, Encoder::mark_internal_failure);
            STATUS_ERROR
        }
    }
}

fn status_code(status: EncoderStatus) -> i32 {
    match status {
        EncoderStatus::NeedMoreInput => STATUS_NEED_MORE_INPUT,
        EncoderStatus::BitstreamAvailable => STATUS_BITSTREAM_AVAILABLE,
        EncoderStatus::EndOfStream => STATUS_END_OF_STREAM,
        EncoderStatus::Error => STATUS_ERROR,
    }
}

fn store_encoder(encoder_result: Result<Encoder, EncoderError>) -> u32 {
    let Ok(encoder) = encoder_result else {
        return 0;
    };
    ENCODERS.with(|encoders| {
        let mut encoders = encoders.borrow_mut();
        if let Some((index, slot)) = encoders
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(encoder);
            u32::try_from(index + 1).unwrap_or(0)
        } else {
            encoders.push(Some(encoder));
            u32::try_from(encoders.len()).unwrap_or(0)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_create() -> u32 {
    store_encoder(Encoder::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_destroy(handle: u32) {
    let index = usize::try_from(handle)
        .ok()
        .and_then(|value| value.checked_sub(1));
    if let Some(index) = index {
        ENCODERS.with(|encoders| {
            if let Some(slot) = encoders.borrow_mut().get_mut(index) {
                *slot = None;
            }
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_alloc(length: u32) -> u32 {
    let Ok(length) = usize::try_from(length) else {
        return 0;
    };
    if length == 0 || length > MAX_WASM_ALLOCATION_BYTES {
        return 0;
    }
    let Ok(layout) = Layout::array::<u8>(length) else {
        return 0;
    };
    // SAFETY: `layout` is a valid non-zero allocation layout and the pointer
    // is returned to JavaScript for exactly one matching deallocation.
    let pointer = unsafe { alloc(layout) };
    if pointer.is_null() || pointer as usize > usize::try_from(u32::MAX).unwrap_or(usize::MAX) {
        if !pointer.is_null() {
            unsafe { dealloc(pointer, layout) };
        }
        return 0;
    }
    let pointer = pointer as usize as u32;
    let tracked = ALLOCATIONS.with(|allocations| {
        let mut allocations = allocations.borrow_mut();
        if allocations.try_reserve(1).is_err() {
            return false;
        }
        allocations.push(WasmAllocation { pointer, length });
        true
    });
    if !tracked {
        unsafe { dealloc(pointer as usize as *mut u8, layout) };
        return 0;
    }
    pointer
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn jocforge_wasm_dealloc(pointer: u32, length: u32) {
    if pointer == 0 {
        return;
    }
    let Ok(length) = usize::try_from(length) else {
        return;
    };
    let Some(allocation) = ALLOCATIONS.with(|allocations| {
        let mut allocations = allocations.borrow_mut();
        allocations
            .iter()
            .position(|allocation| allocation.pointer == pointer && allocation.length == length)
            .map(|index| allocations.remove(index))
    }) else {
        return;
    };
    let Ok(layout) = Layout::array::<u8>(allocation.length) else {
        return;
    };
    // SAFETY: The JavaScript wrapper passes the exact pointer and length
    // returned by jocforge_wasm_alloc.
    unsafe { dealloc(pointer as usize as *mut u8, layout) };
}

fn known_allocation(pointer: u32, length: usize) -> bool {
    ALLOCATIONS.with(|allocations| {
        allocations
            .borrow()
            .iter()
            .any(|allocation| allocation.pointer == pointer && allocation.length == length)
    })
}

fn wasm_memory_range_valid(pointer: u32, length: usize) -> bool {
    let Some(end) = usize::try_from(pointer)
        .ok()
        .and_then(|start| start.checked_add(length))
    else {
        return false;
    };
    #[cfg(target_arch = "wasm32")]
    {
        let pages = usize::try_from(core::arch::wasm32::memory_size(0)).unwrap_or(0);
        return end <= pages.saturating_mul(65_536);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = end;
        true
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn jocforge_wasm_encoder_push_bytes(
    handle: u32,
    pointer: u32,
    length: u32,
) -> i32 {
    let Ok(length) = usize::try_from(length) else {
        return STATUS_ERROR;
    };
    if length != 0 && (pointer == 0 || !known_allocation(pointer, length)) {
        return STATUS_ERROR;
    }
    if !wasm_memory_range_valid(pointer, length) {
        return STATUS_ERROR;
    }
    // SAFETY: the allocation registry confirms this exact live region.
    let bytes = unsafe { slice::from_raw_parts(pointer as usize as *const u8, length) };
    guarded_status(handle, |encoder| encoder.push_bytes(bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_dependent_count(handle: u32, dependent_count: u32) {
    let _ = with_encoder(handle, |encoder| {
        encoder.set_dependent_count(u8::try_from(dependent_count).unwrap_or(1));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_layout(handle: u32, layout: u32) {
    let _ = with_encoder(handle, |encoder| {
        encoder.set_layout(u8::try_from(layout).unwrap_or(0));
    });
}

/// Selects the native flat-7.1 Surround EX declaration: 0 = unspecified,
/// 1 = not encoded, 2 = encoded.
#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_surround_ex(handle: u32, surround_ex: u32) {
    let _ = with_encoder(handle, |encoder| {
        encoder.set_surround_ex(u8::try_from(surround_ex).unwrap_or(1));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_job_xml(handle: u32, pointer: u32, length: u32) -> i32 {
    if length == 0 || length > MAX_WASM_ALLOCATION_BYTES as u32 {
        return STATUS_ERROR;
    }
    if pointer == 0 {
        return STATUS_ERROR;
    }
    let bytes = unsafe { slice::from_raw_parts(pointer as usize as *const u8, length as usize) };
    match with_encoder(handle, |encoder| encoder.set_job_xml(bytes.to_vec())) {
        Some(()) => 0,
        None => STATUS_ERROR,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_rate(handle: u32, rate_kbps: u32) {
    let _ = with_encoder(handle, |encoder| {
        encoder.set_rate_kbps(rate_kbps);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_set_objects(handle: u32, object_count: u32) {
    let _ = with_encoder(handle, |encoder| {
        encoder.set_object_count(u8::try_from(object_count).unwrap_or(15));
    });
}

/// Encodes the host-streamed ADM; the input is served from the worker's OPFS handle.
#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_finish_streamed(handle: u32) -> i32 {
    guarded_status(handle, |encoder| match encoder.finish_streamed() {
        Ok(status) => status,
        Err(_) => EncoderStatus::Error,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_finish(handle: u32) -> i32 {
    match catch_unwind(AssertUnwindSafe(|| with_encoder(handle, Encoder::finish))) {
        Ok(Some(Ok(status))) => with_encoder(handle, |encoder| {
            if encoder.last_error().is_some() {
                STATUS_ERROR
            } else {
                status_code(status)
            }
        })
        .unwrap_or(STATUS_ERROR),
        Ok(Some(Err(_))) => STATUS_ERROR,
        Ok(None) | Err(_) => {
            let _ = with_encoder(handle, Encoder::mark_internal_failure);
            STATUS_ERROR
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_bitstream_ptr(handle: u32) -> u32 {
    with_encoder(handle, |encoder| {
        encoder
            .expose_bitstream()
            .map_or(0, |bytes| bytes.as_ptr() as usize as u32)
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_bitstream_len(handle: u32) -> u32 {
    with_encoder(handle, |encoder| {
        encoder
            .expose_bitstream()
            .map_or(0, |bytes| u32::try_from(bytes.len()).unwrap_or(u32::MAX))
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_error_ptr(handle: u32) -> u32 {
    with_encoder(handle, |encoder| {
        encoder
            .last_error()
            .map_or(0, |error| error.detail.as_ptr() as usize as u32)
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_error_len(handle: u32) -> u32 {
    with_encoder(handle, |encoder| {
        encoder.last_error().map_or(0, |error| {
            u32::try_from(error.detail.len()).unwrap_or(u32::MAX)
        })
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn jocforge_wasm_encoder_error_category(handle: u32) -> u32 {
    with_encoder(handle, |encoder| {
        encoder.error_category_code().unwrap_or(u32::MAX)
    })
    .unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        jocforge_wasm_encoder_create, jocforge_wasm_encoder_destroy,
        jocforge_wasm_encoder_error_category, jocforge_wasm_encoder_finish,
        jocforge_wasm_encoder_push_bytes,
    };

    #[test]
    fn raw_abi_rejects_invalid_handles_and_untracked_input() {
        assert_eq!(jocforge_wasm_encoder_finish(0), -1);
        assert_eq!(jocforge_wasm_encoder_error_category(0), u32::MAX);
        let handle = jocforge_wasm_encoder_create();
        assert_ne!(handle, 0);
        let status = unsafe { jocforge_wasm_encoder_push_bytes(handle, 0x1000, 64) };
        assert_eq!(status, -1);
        jocforge_wasm_encoder_destroy(handle);
        assert_eq!(jocforge_wasm_encoder_finish(handle), -1);
    }
}

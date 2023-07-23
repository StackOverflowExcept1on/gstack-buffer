#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ffi::c_void;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::slice::from_raw_parts_mut;

const MAX_BUFFER_SIZE: usize = 64 * 1024;

type Callback = unsafe extern "C" fn(ptr: *mut u8, data: *mut c_void);

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn c_with_alloca(size: usize, callback: Callback, data: *mut c_void);
}

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" fn c_with_alloca(_size: usize, callback: Callback, data: *mut c_void) {
    let mut buffer = MaybeUninit::<[MaybeUninit<u8>; MAX_BUFFER_SIZE]>::uninit().assume_init();
    callback(buffer.as_mut_ptr() as *mut _, data);
}

fn with_alloca<T>(size: usize, f: impl FnOnce(&mut [MaybeUninit<u8>]) -> T) -> T {
    #[inline(always)]
    fn get_trampoline<F: FnMut(*mut u8)>(_closure: &F) -> Callback {
        trampoline::<F>
    }

    unsafe extern "C" fn trampoline<F: FnMut(*mut u8)>(ptr: *mut u8, data: *mut c_void) {
        let f = &mut *(data as *mut F);
        f(ptr);
    }

    let mut f = ManuallyDrop::new(f);
    let mut ret = MaybeUninit::uninit();

    let mut closure = |ptr| unsafe {
        let slice = from_raw_parts_mut(ptr as *mut _, size);
        ret.write(ManuallyDrop::take(&mut f)(slice));
    };

    let trampoline = get_trampoline(&closure);

    unsafe {
        c_with_alloca(size, trampoline, &mut closure as *mut _ as *mut c_void);
        ret.assume_init()
    }
}

pub fn with_byte_buffer<T>(size: usize, f: impl FnOnce(&mut [MaybeUninit<u8>]) -> T) -> T {
    #[cfg(feature = "alloc")]
    if size > MAX_BUFFER_SIZE {
        return f(alloc::vec::Vec::with_capacity(size).spare_capacity_mut());
    }
    with_alloca(size, f)
}
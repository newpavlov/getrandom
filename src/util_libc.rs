// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::util::LazyUsize;
use core::ptr::NonNull;

// A "weak" binding to a C function that may or may not be present at runtime.
// Used for supporting newer OS features while still building on older systems.
// F must be a function pointer of type `unsafe extern "C" fn`. Based off of the
// weak! macro in libstd.
pub struct Weak {
    name: &'static str,
    addr: LazyUsize,
}

impl Weak {
    // Construct a binding to a C function with a given name. This function is
    // unsafe because `name` _must_ be null terminated.
    pub const unsafe fn new(name: &'static str) -> Self {
        Self {
            name,
            addr: LazyUsize::new(),
        }
    }

    // Return a function pointer if present at runtime. Otherwise, return null.
    pub fn ptr(&self) -> Option<NonNull<libc::c_void>> {
        let addr = self.addr.unsync_init(|| unsafe {
            libc::dlsym(libc::RTLD_DEFAULT, self.name.as_ptr() as *const _) as usize
        });
        NonNull::new(addr as *mut _)
    }
}

pub struct LazyFd(LazyUsize);

impl LazyFd {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    // If init() returns Some(x), x should be nonnegative.
    pub fn init(&self, init: impl FnOnce() -> Option<libc::c_int>) -> Option<libc::c_int> {
        let fd = self.0.sync_init(
            || match init() {
                // OK as val >= 0 and val <= c_int::MAX < usize::MAX
                Some(val) => val as usize,
                None => LazyUsize::UNINIT,
            },
            || unsafe {
                libc::usleep(1000);
            },
        );
        match fd {
            LazyUsize::UNINIT => None,
            val => Some(val as libc::c_int),
        }
    }
}

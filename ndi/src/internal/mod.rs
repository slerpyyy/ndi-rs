#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::{
    ops::{Deref, DerefMut},
    sync::RwLock,
};

#[cfg(target_os = "windows")]
mod bindings_windows;

#[cfg(target_os = "macos")]
mod bindings_macos;

#[cfg(target_os = "linux")]
mod bindings_linux;

pub mod bindings {
    #[cfg(target_os = "windows")]
    pub use super::bindings_windows::*;

    #[cfg(target_os = "macos")]
    pub use super::bindings_macos::*;

    #[cfg(target_os = "linux")]
    pub use super::bindings_linux::*;
}

lazy_static::lazy_static! {
    pub(crate) static ref LIBRARY_SINGLETON: RwLock<Option<bindings::NdiDynLib>> = RwLock::new(None);
}

pub(crate) fn lib_ref() -> &'static RwLock<Option<bindings::NdiDynLib>> {
    &LIBRARY_SINGLETON
}

pub(crate) unsafe fn load(path: impl AsRef<std::ffi::OsStr>) -> Result<(), libloading::Error> {
    let mut lib = lib_ref().write().unwrap();
    *lib = Some(bindings::NdiDynLib::new(path)?);
    Ok(())
}

macro_rules! lib_unwrap {
    () => {
        crate::internal::lib_ref()
            .read()
            .unwrap()
            .as_ref()
            .expect("Expected library to be loaded.")
    };
}

pub(crate) use lib_unwrap;

/// Utility for adding a destructor function to a pointer which is called once the struct is dropped.
pub(crate) struct OnDrop<P: Copy> {
    inner: P,
    destroy: fn(P),
}

impl<T> OnDrop<*mut T> {
    pub(crate) fn new(inner: *mut T, destroy: fn(*mut T)) -> Self {
        OnDrop { inner, destroy }
    }
}

impl<P: Copy> Deref for OnDrop<P> {
    type Target = P;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<P: Copy> DerefMut for OnDrop<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<P: Copy> Drop for OnDrop<P> {
    fn drop(&mut self) {
        (self.destroy)(self.inner);
    }
}

#[test]
fn on_drop_simple() {
    fn drop_boxed_i32(b: *mut i32) {
        let b = unsafe { Box::from_raw(b) };
        assert_eq!(*b, 5);
        drop(b);
    }

    let num = Box::into_raw(Box::new(2));
    unsafe { *num += 3 };

    let on_drop = OnDrop::new(num, drop_boxed_i32);
    drop(on_drop);
}

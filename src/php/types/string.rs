//! Represents a string in the PHP world. Similar to a C string, but is reference counted and
//! contains the length of the string, meaning the string can contain the NUL character.

use core::slice;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
};

use crate::{
    bindings::{
        ext_php_rs_zend_string_init, ext_php_rs_zend_string_release, zend_string,
        zend_string_init_interned,
    },
    errors::{Error, Result},
    functions::c_str,
};

/// A wrapper around the [`zend_string`] used within the Zend API. Essentially a C string, except
/// that the structure contains the length of the string as well as the string being refcounted.
pub struct ZendString {
    ptr: *mut zend_string,
    free: bool,
}

impl ZendString {
    /// Creates a new Zend string.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    /// * `persistent` - Whether the request should relive the request boundary.
    pub fn new(str_: impl AsRef<str>, persistent: bool) -> Self {
        let str_ = str_.as_ref();

        Self {
            ptr: unsafe { ext_php_rs_zend_string_init(c_str(str_), str_.len() as _, persistent) },
            free: true,
        }
    }

    /// Creates a new interned Zend string.
    ///
    /// # Parameters
    ///
    /// * `str_` - The string to create a Zend string from.
    pub fn new_interned(str_: impl AsRef<str>) -> Self {
        let str_ = str_.as_ref();

        Self {
            ptr: unsafe { zend_string_init_interned.unwrap()(c_str(str_), str_.len() as _, true) },
            free: true,
        }
    }

    /// Creates a new [`ZendString`] wrapper from a raw pointer to a [`zend_string`].
    ///
    /// # Parameters
    ///
    /// * `ptr` - A raw pointer to a [`zend_string`].
    /// * `free` - Whether the pointer should be freed when the resulting [`ZendString`] goes
    /// out of scope.
    ///
    /// # Safety
    ///
    /// As a raw pointer is given this function is unsafe, you must ensure the pointer is valid when calling
    /// the function. A simple null check is done but this is not sufficient in most places.
    pub unsafe fn from_ptr(ptr: *mut zend_string, free: bool) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        Some(Self { ptr, free })
    }

    /// Releases the Zend string, returning the raw pointer to the `zend_string` object
    /// and consuming the internal Rust [`NewZendString`] container.
    pub fn release(mut self) -> *mut zend_string {
        self.free = false;
        self.ptr
    }

    /// Borrows the underlying internal pointer of the Zend string.
    pub(crate) fn borrow_ptr(&self) -> *mut zend_string {
        self.ptr
    }
}

impl Drop for ZendString {
    fn drop(&mut self) {
        if self.free && !self.ptr.is_null() {
            unsafe { ext_php_rs_zend_string_release(self.ptr) };
        }
    }
}

impl TryFrom<ZendString> for String {
    type Error = Error;

    fn try_from(value: ZendString) -> Result<Self> {
        <String as TryFrom<&ZendString>>::try_from(&value)
    }
}

impl TryFrom<&ZendString> for String {
    type Error = Error;

    fn try_from(s: &ZendString) -> Result<Self> {
        let zs = unsafe { s.ptr.as_ref() }.ok_or(Error::InvalidPointer)?;

        // SAFETY: Zend strings have a length that we know we can read.
        // By reading this many bytes we will not run into any issues.
        //
        // We can safely cast our *const c_char into a *const u8 as both
        // only occupy one byte.
        std::str::from_utf8(unsafe {
            slice::from_raw_parts(zs.val.as_ptr() as *const u8, zs.len as _)
        })
        .map(|s| s.to_string())
        .map_err(|_| Error::InvalidPointer)
    }
}

impl Debug for ZendString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Result<String> = self.try_into();
        match s {
            Ok(s) => s.fmt(f),
            Err(_) => Option::<()>::None.fmt(f),
        }
    }
}
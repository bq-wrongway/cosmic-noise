//! The [`ctor`] crate reimplemented using procedural macros.
//!
//! [`ctor`]: https://crates.io/crates/ctor
//!
//! In some cases it is necessary to run code at the very start or the very end
//! of the program. This crate provides a macro that can be used to run code at
//! the very beginning of program execution, along with some extra features.
//!
//! ## Advantages over [`ctor`]
//!
//! - Completely dependency free, thanks to relying on procedural macros instead
//!   of proc macros.
//! - Supports all of the same use cases as the [`ctor`] crate.
//! - Supports all of the same platforms as the [`ctor`] crate.
//! - Fixes a couple of warts in [`ctor`]'s API, such as:
//!   - `unsafe` is required when it is used, see the "Safety" section below.
//!   - Global variables are required to be `Sync`.
//!   - Global variables use `MaybeUninit` instead of `Option`.
//!   - Functions set up with the `ctor` or `dtor` macros cannot be called in
//!     other Rust code.
//!
//! ## Disadvantages
//!
//! - The API has a slightly different form factor that can be inconvenient in
//!   some cases.
//! - The MSRV has been raised to 1.36.0.
//!
//! ## Functional Usage
//!
//! The `ctor` macro can be used to run a function at program startup time.
//!
//! ```
//! use std::sync::atomic::{AtomicUsize, Ordering};
//!
//! static INITIALIZED: AtomicUsize = AtomicUsize::new(0);
//!
//! ctor_lite::ctor! {
//!     unsafe fn set_value() {
//!         INITIALIZED.store(1, Ordering::Relaxed);
//!     }
//! }
//!
//! assert_eq!(INITIALIZED.load(Ordering::Relaxed), 1);
//! ```
//!
//! Note that this macro is a procedural block rather than an attribute macro.
//! If you prefer the old way of using the macro you can use the
//! [`macro-rules-attribute`] crate.
//!
//! [`macro-rules-attribute`]: https://crates.io/crates/macro-rules-attribute
//!
//! ```
//! use macro_rules_attribute::apply;
//! use std::sync::atomic::{AtomicUsize, Ordering};
//!
//! static INITIALIZED: AtomicUsize = AtomicUsize::new(0);
//!
//! #[apply(ctor_lite::ctor!)]
//! unsafe fn set_value() {
//!     INITIALIZED.store(1, Ordering::Relaxed);
//! }
//!
//! assert_eq!(INITIALIZED.load(Ordering::Relaxed), 1);
//! ```
//!
//! ## Static Usage
//!
//! The `ctor` macro can be used to create a static variable initialized to a
//! default value. At startup time, the function is used to initialize the
//! static variable.
//!
//! ```
//! fn value() -> i32 {
//!     6
//! }
//!
//! ctor_lite::ctor! {
//!     unsafe static VALUE: i32 = value();
//! }
//!
//! assert_eq!(*VALUE, 6);
//! ```
//!
//! ## Destructor
//!
//! This crate can also be used to run a function at program exit as well. The
//! `dtor` macro can be used to run a function when the program ends.
//!
//! ```
//! use macro_rules_attribute::apply;
//!
//! #[apply(ctor_lite::dtor!)]
//! unsafe fn run_at_exit() {
//!     do_some_cleanup();
//! }
//!
//! # fn do_some_cleanup() {}
//! ```
//!
//! ## Safety
//!
//! Macros from this crate must be used with care. In general Rust code is run
//! with the assumption that no other code is run before program startup, and
//! no other code is run after program shutdown. Specifically, `libstd` sets up
//! some global variables before the `main` function and then assumes these
//! variables are set throughout its runtime. Therefore, calling `libstd`
//! functions that use these variables will lead to undefined behavior.
//!
//! Generally, functions from `core` or `alloc` are safe to call in these
//! functions. In addition, functions from [`libc`] should be able to be called
//! freely, as well as most of the functions contained in [`rustix`]. Other
//! crates should be used only when it is understood what other calls they
//! contain.
//!
//! [`libc`]: https://crates.io/crates/libc
//! [`rustix`]: https://crates.io/crates/rustix
//!
//! In addition, no ordering is guaranteed for functions ran in the `ctor` or
//! `dtor` macros.
//!
//! ## Implementation
//!
//! The `ctor` macro works by creating a function with linker attributes that
//! place it into a special section in the file. When the C runtime starts the
//! program, it reads function pointers from this section and runs them.
//!
//! This function call...
//!
//! ```
//! ctor_lite::ctor! {
//!     unsafe fn foo() { /* ... */ }
//! }
//! ```
//!
//! ...is translated to code that looks like this:
//!
//! ```
//! #[used]
//! #[cfg_attr(any(target_os = "linux", target_os = "android"), link_section = ".init_array")]
//! #[cfg_attr(target_os = "freebsd", link_section = ".init_array")]
//! #[cfg_attr(target_os = "netbsd", link_section = ".init_array")]
//! #[cfg_attr(target_os = "openbsd", link_section = ".init_array")]
//! #[cfg_attr(target_os = "illumos", link_section = ".init_array")]
//! #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = "__DATA_CONST,__mod_init_func")]
//! #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
//! static FOO: extern fn() = {
//!   #[cfg_attr(any(target_os = "linux", target_os = "android"), link_section = ".text.startup")]
//!   extern fn foo() { /* ... */ };
//!   foo
//! };
//! ```
//!
//! When creating a global constant with the `ctor` macro it writes code that
//! runs the function then writes the value into a global constant.
//!
//! This code...
//!
//! ```
//! ctor_lite::ctor! {
//!     unsafe static FOO: i32 = foo();
//! }
//! # fn foo() -> i32 { 1 }
//! ```
//!
//! ...is translated to code that looks like this, with modifications that allow
//! for `FOO` to be used from safe code:
//!
//! ```no_compile
//! static mut FOO: i32 = core::mem::uninitialized();
//! ctor_lite::ctor! {
//!     unsafe fn init_storage() {
//!         FOO = foo();
//!     }
//! }
//! # fn foo() -> i32 { 1 }
//! ```
//!
//! When functions are put into `dtor`, it runs `ctor` with the `libc::atexit`
//! function to ensure that the function is run at program exit.
//!
//! This code...
//!
//! ```
//! ctor_lite::dtor! {
//!     unsafe fn foo() {
//!         /* ... */
//!     }
//! }
//! ```
//!
//! ...is translated to code that looks like this, with modifications that let
//! us avoid a dependency on the [`libc`] crate:
//!
//! ```no_compile
//! unsafe fn foo() {
//!     /* ... */
//! }
//!
//! ctor_lite::ctor! {
//!     unsafe fn run_dtor() {
//!         libc::atexit(foo);
//!     }
//! }
//! ```

#![no_std]

/// Run a function on program startup or initialize a constant.
///
/// See the crate level documentation for more info.
#[macro_export]
macro_rules! ctor {
    // Case 1: Run a function at startup time.
    (
        $(#[$meta:meta])*
        $vis:vis unsafe fn $name:ident () $bl:block
    ) => {
        const _: () = {
            $(#[$meta])*
            $vis unsafe fn $name () {
                unsafe fn __this_thing_is_always_unsafe() {}
                __this_thing_is_always_unsafe();
                $bl
            }

            #[cfg(not(any(
                target_os = "linux",
                target_os = "android",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "dragonfly",
                target_os = "illumos",
                target_os = "haiku",
                target_os = "macos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "tvos",
                windows
            )))]
            compile_error!("ctor! is not supported on the current target");

            #[used]
            #[allow(non_upper_case_globals, non_snake_case)]
            #[doc(hidden)]
            #[cfg_attr(
                any(target_os = "linux", target_os = "android"),
                link_section = ".init_array"
            )]
            #[cfg_attr(target_os = "freebsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "netbsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "openbsd", link_section = ".init_array")]
            #[cfg_attr(target_os = "dragonfly", link_section = ".init_array")]
            #[cfg_attr(target_os = "illumos", link_section = ".init_array")]
            #[cfg_attr(target_os = "haiku", link_section = ".init_array")]
            #[cfg_attr(
                any(
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "visionos",
                    target_os = "tvos"
                ),
                link_section = "__DATA,__mod_init_func"
            )]
            #[cfg_attr(windows, link_section = ".CRT$XCU")]
            static __rust_ctor_lite__ctor: unsafe extern "C" fn() -> usize = {
                #[cfg_attr(
                    any(target_os = "linux", target_os = "android"),
                    link_section = ".text.startup"
                )]
                unsafe extern "C" fn ctor() -> usize {
                    $name ();
                    0
                }

                ctor
            };
        };
    };

    // Case 2: Initialize a constant at bootup time.
    (
        $(#[$meta:meta])*
        $vis:vis unsafe static $(mut)? $name:ident:$ty:ty = $e:expr;
    ) => {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        $vis struct $name<T> {
            _data: ::core::marker::PhantomData<T>
        }

        $(#[$meta:meta])*
        $vis static $name: $name<$ty> = $name {
            _data: ::core::marker::PhantomData::<$ty>
        };

        const _: () = {
            use ::core::cell::UnsafeCell;
            use ::core::mem::MaybeUninit;
            use ::core::ops::Deref;

            struct SyncSlot(UnsafeCell<MaybeUninit<$ty>>);
            unsafe impl Sync for SyncSlot {}

            static STORAGE: SyncSlot = {
                SyncSlot(UnsafeCell::new(MaybeUninit::uninit()))
            };

            impl Deref for $name<$ty> {
                type Target = $ty;

                fn deref(&self) -> &$ty {
                    // SAFETY: This will always be initialized.
                    unsafe {
                        &*(&*STORAGE.0.get()).as_ptr()
                    }
                }
            }

            $crate::ctor! {
                unsafe fn init_storage() {
                    let val = $e;

                    // SAFETY: We are the only ones who can write into STORAGE.
                    unsafe {
                        *STORAGE.0.get() = MaybeUninit::new(val);
                    }
                }
            }

            fn __assert_type_is_sync() {
                fn __must_be_sync<T: Sync>() {}
                __must_be_sync::<$ty>();
            }
        };
    }
}

/// Run a function on program shutdown.
///
/// See the crate level documentation for more information.
#[macro_export]
macro_rules! dtor {
    (
        $(#[$meta:meta])*
        $vis:vis unsafe fn $name:ident () $bl:block
    ) => {
        const _: () = {
            $(#[$meta])*
            $vis unsafe fn $name () {
                unsafe fn __this_thing_is_always_unsafe() {}
                __this_thing_is_always_unsafe();
                $bl
            }

            // Link directly to atexit in order to avoid a libc dependency.
            #[cfg(not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "tvos"
            )))]
            #[inline(always)]
            unsafe fn __do_atexit(cb: unsafe extern fn()) {
                extern "C" {
                    fn atexit(cb: unsafe extern fn());
                }
                atexit(cb);
            }

            // For platforms that have __cxa_atexit, we register the dtor as scoped to dso_handle
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "tvos"
            ))]
            #[inline(always)]
            unsafe fn __do_atexit(cb: unsafe extern fn(_: *const u8)) {
                extern "C" {
                    static __dso_handle: *const u8;
                    fn __cxa_atexit(
                        cb: unsafe extern fn(_: *const u8),
                        arg: *const u8,
                        dso_handle: *const u8
                    );
                }
                __cxa_atexit(cb, ::core::ptr::null(), __dso_handle);
            }

            #[cfg(not(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "tvos"
            )))]
            #[cfg_attr(
                any(
                    target_os = "linux",
                    target_os = "android"
                ),
                link_section = ".text.exit"
            )]
            unsafe extern "C" fn __run_destructor() { $name() };
            #[cfg(any(
                target_os = "macos",
                target_os = "ios",
                target_os = "visionos",
                target_os = "tvos"
            ))]
            unsafe extern "C" fn __run_destructor(_: *const u8) { $name() };

            $crate::ctor! {
                unsafe fn register_dtor() {
                    __do_atexit(__run_destructor);
                }
            }
        };
    };
}

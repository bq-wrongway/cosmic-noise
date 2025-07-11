// SPDX-License-Identifier: MIT OR Apache-2.0 OR Zlib

// Copyright 2023 John Nunley
//
// Licensed under the Apache License, Version 2.0, the MIT License, and
// the Zlib license. You may not use this software except in compliance
// with at least one of these licenses. You should have received a copy
// of these licenses with this software. You may also find them at:
//
//     http://www.apache.org/licenses/LICENSE-2.0
//     https://opensource.org/licenses/MIT
//     https://opensource.org/licenses/Zlib
//
// Unless required by applicable law or agreed to in writing, software
// distributed under these licenses is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the licenses for the specific language governing permissions and
// limitations under the licenses.

//! A tiny set of bindings to the [Xlib] library.
//!
//! The primary contemporary library for handling [Xlib] is the [`x11-dl`] crate. However, there are three
//! primary issues.
//!
//! 1. **You should not be using Xlib in 2023.** [Xlib] is legacy code, and even that doesn't get across
//!     how poor the API decisions that it's locked itself into are. It has a global error hook for
//!     some reason, thread-safety is a mess, and it has so many soundness holes it might as well be made
//!     out of swiss cheese. You should not be using [Xlib]. If you *have* to use [Xlib], you should just
//!     run all of your logic using the much more sound [XCB] library, or, even more ideally, something
//!     like [`x11rb`]. Then, you take the `Display` pointer and use it for whatever legacy API you've
//!     locked yourself into, and use [XCB] or [`x11rb`] for everything else. Yes, I just called [GLX]
//!     a legacy API. It's the 2020's now. [Vulkan] and [`wgpu`] run everywhere aside from legacy machines.
//!     Not to mention, they support [XCB].
//!
//! 2. Even if you manage to use [`x11-dl`] without tripping over the legacy API, it is a massive crate.
//!     [Xlib] comes with quite a few functions, most of which are unnecessary in the 21st century.
//!     Even if you don't use any of these and just stick to [XCB], you still pay the price for it.
//!     Binaries that use [`x11-dl`] need to dedicate a significant amount of their binary and memory
//!     space to the library. Even on Release builds, I have recorded [`x11-dl`] taking up to seven
//!     percent of the binary.
//!
//! 3. Global error handling. [Xlib] has a single global error hook. This is reminiscent of the Unix
//!     signal handling API, in that it makes it difficult to create well-modularized programs
//!     since they will fight with each-other over the error handlers. However, unlike the signal
//!     handling API, there is no way to tell if you're replacing an existing error hook.
//!
//! `tiny-xlib` aims to solve all of these problems. It provides a safe API around [Xlib] that is
//! conducive to being handed off to both [XCB] APIs and legacy [Xlib] APIs. The library only
//! imports absolutely necessary functions. In addition, it also provides a common API for
//! handling errors in a safe, modular way.
//!
//! # Features
//!
//! - Safe API around [Xlib]. See the [`Display`] structure.
//! - Minimal set of dependencies.
//! - Implements [`AsRawXcbConnection`], which allows it to be used with [XCB] APIs.
//! - Modular error handling.
//!
//! # Non-Features
//!
//! - Any API outside of opening [`Display`]s and handling errors. If this library doesn't support some
//!   feature, it's probably intentional. You should use [XCB] or [`x11rb`] instead. This includes:
//!  - Window management.
//!  - Any extensions outside of `Xlib-xcb`.
//!  - IME handling.
//!  - Hardware rendering.
//!
//! # Examples
//!
//! ```no_run
//! use as_raw_xcb_connection::AsRawXcbConnection;
//! use tiny_xlib::Display;
//!
//! use x11rb::connection::Connection;
//! use x11rb::xcb_ffi::XCBConnection;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a display.
//! let display = Display::new(None)?;
//!
//! // Get the XCB connection.
//! let xcb_conn = display.as_raw_xcb_connection();
//!
//! // Use that pointer to create a new XCB connection.
//! let xcb_conn = unsafe {
//!     XCBConnection::from_raw_xcb_connection(xcb_conn.cast(), false)?
//! };
//!
//! // Register a handler for X11 errors.
//! tiny_xlib::register_error_handler(Box::new(|_, error| {
//!     println!("X11 error: {:?}", error);
//!     false
//! }));
//!
//! // Do whatever you want with the XCB connection.
//! loop {
//!     println!("Event: {:?}", xcb_conn.wait_for_event()?);
//! }
//! # Ok(()) }
//! ```
//!
//! # Optional Features
//!
//! - `tracing`, enabled by default, enables telemetry using the [`tracing`] crate.
//! - `dlopen` uses the [`libloading`] library to load the X11 libraries instead of linking to them
//!   directly.
//!
//! [Xlib]: https://en.wikipedia.org/wiki/Xlib
//! [XCB]: https://xcb.freedesktop.org/
//! [`x11-dl`]: https://crates.io/crates/x11-dl
//! [`x11rb`]: https://crates.io/crates/x11rb
//! [GLX]: https://en.wikipedia.org/wiki/GLX
//! [Vulkan]: https://www.khronos.org/vulkan/
//! [`wgpu`]: https://crates.io/crates/wgpu
//! [`Display`]: struct.Display.html
//! [`AsRawXcbConnection`]: https://docs.rs/as_raw_xcb_connection/latest/as_raw_xcb_connection/trait.AsRawXcbConnection.html
//! [`tracing`]: https://crates.io/crates/tracing
//! [`libloading`]: https://crates.io/crates/libloading

#![allow(unused_unsafe)]
#![cfg_attr(coverage, feature(no_coverage))]

mod ffi;

use std::cell::Cell;
use std::ffi::CStr;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::os::raw::{c_int, c_void};
use std::ptr::{self, NonNull};
use std::sync::{Mutex, MutexGuard, Once, PoisonError};

macro_rules! lock {
    ($e:expr) => {{
        // Make sure this isn't flagged with coverage.
        #[cfg_attr(coverage, no_coverage)]
        fn unwrapper<T>(guard: PoisonError<MutexGuard<'_, T>>) -> MutexGuard<'_, T> {
            guard.into_inner()
        }

        ($e).lock().unwrap_or_else(unwrapper)
    }};
}

ctor_lite::ctor! {
    unsafe static XLIB: io::Result<ffi::Xlib> = {
        #[cfg_attr(coverage, no_coverage)]
        unsafe fn load_xlib_with_error_hook() -> io::Result<ffi::Xlib> {
            // Here's a puzzle: how do you *safely* add an error hook to Xlib? Like signal handling, there
            // is a single global error hook. Therefore, we need to make sure that we economize on the
            // single slot that we have by offering a way to set it. However, unlike signal handling, there
            // is no way to tell if we're replacing an existing error hook. If we replace another library's
            // error hook, we could cause unsound behavior if it assumes that it is the only error hook.
            //
            // However, we don't want to call the default error hook, because it exits the program. So, in
            // order to tell if the error hook is the default one, we need to compare it to the default
            // error hook. However, we can't just compare the function pointers, because the default error
            // hook is a private function that we can't access.
            //
            // In order to access it, before anything else runs, this function is called. It loads Xlib,
            // sets the error hook to a dummy function, reads the resulting error hook into a static
            // variable, and then resets the error hook to the default function. This allows us to read
            // the default error hook and compare it to the one that we're setting.
            #[cfg_attr(coverage, no_coverage)]
            fn error(e: impl std::error::Error) -> io::Error {
                io::Error::new(io::ErrorKind::Other, format!("failed to load Xlib: {}", e))
            }
            let xlib = ffi::Xlib::load().map_err(error)?;

            // Dummy function we use to set the error hook.
            #[cfg_attr(coverage, no_coverage)]
            unsafe extern "C" fn dummy(
                _display: *mut ffi::Display,
                _error: *mut ffi::XErrorEvent,
            ) -> std::os::raw::c_int {
                0
            }

            // Set the error hook to the dummy function.
            let default_hook = xlib.set_error_handler(Some(dummy));

            // Read the error hook into a static variable.
            // SAFETY: This should only run once at the start of the program, no need to worry about
            // multithreading.
            DEFAULT_ERROR_HOOK.set(default_hook);

            // Set the error hook back to the default function.
            xlib.set_error_handler(default_hook);

            Ok(xlib)
        }

        unsafe { load_xlib_with_error_hook() }
    };
}

#[inline]
fn get_xlib(sym: &io::Result<ffi::Xlib>) -> io::Result<&ffi::Xlib> {
    // Eat coverage on the error branch.
    #[cfg_attr(coverage, no_coverage)]
    fn error(e: &io::Error) -> io::Error {
        io::Error::new(e.kind(), e.to_string())
    }

    sym.as_ref().map_err(error)
}

/// The default error hook to compare against.
static DEFAULT_ERROR_HOOK: ErrorHookSlot = ErrorHookSlot::new();

/// An error handling hook.
type ErrorHook = Box<dyn FnMut(&Display, &ErrorEvent) -> bool + Send + Sync + 'static>;

/// List of error hooks to invoke.
static ERROR_HANDLERS: Mutex<HandlerList> = Mutex::new(HandlerList::new());

/// Global error handler for X11.
unsafe extern "C" fn error_handler(
    display: *mut ffi::Display,
    error: *mut ffi::XErrorEvent,
) -> c_int {
    // Abort the program if the error hook panics.
    struct AbortOnPanic;
    impl Drop for AbortOnPanic {
        #[cfg_attr(coverage, no_coverage)]
        #[cold]
        #[inline(never)]
        fn drop(&mut self) {
            std::process::abort();
        }
    }

    let bomb = AbortOnPanic;

    let mut handlers = lock!(ERROR_HANDLERS);

    let prev = handlers.prev;
    if let Some(prev) = prev {
        // Drop the mutex lock to make sure no deadlocks occur. Otherwise, if the prev handlers
        // tries to add its own handler, we'll deadlock.
        drop(handlers);

        unsafe {
            // Run the previous error hook, if any.
            prev(display, error);
        }

        // Restore the mutex lock.
        handlers = lock!(ERROR_HANDLERS);
    }

    // Read out the variables.
    // SAFETY: Guaranteed to be a valid display setup.
    let display_ptr = unsafe { Display::from_ptr(display.cast()) };
    let event = ErrorEvent(ptr::read(error));

    #[cfg(feature = "tracing")]
    tracing::error!(
        display = ?&*display_ptr,
        error = ?event,
        "got Xlib error",
    );

    // Invoke the error hooks.
    handlers.iter_mut().any(|(_i, handler)| {
        #[cfg(feature = "tracing")]
        tracing::trace!(key = _i, "invoking error handler");

        let stop_going = (handler)(&display_ptr, &event);

        #[cfg(feature = "tracing")]
        {
            if stop_going {
                tracing::trace!("error handler returned true, stopping");
            } else {
                tracing::trace!("error handler returned false, continuing");
            }
        }

        stop_going
    });

    // Defuse the bomb.
    mem::forget(bomb);

    // Apparently the return value here has no effect.
    0
}

/// Register the error handler.
fn setup_error_handler(xlib: &ffi::Xlib) {
    static REGISTERED: Once = Once::new();
    REGISTERED.call_once(move || {
        // Make sure threads are initialized here.
        unsafe {
            xlib.init_threads();
        }

        // Get the previous error handler.
        let prev = unsafe { xlib.set_error_handler(Some(error_handler)) };

        // If it isn't the default error handler, then we need to store it.
        // SAFETY: DEFAULT_ERROR_HOOK is not set after the program starts, so this is safe.
        let default_hook = unsafe { DEFAULT_ERROR_HOOK.get() };
        if prev != default_hook.flatten() && prev != Some(error_handler) {
            lock!(ERROR_HANDLERS).prev = prev;
        }
    });
}

/// A key to the error handler list that can be used to remove handlers.
#[derive(Debug, Copy, Clone)]
pub struct HandlerKey(usize);

/// The error event type.
#[derive(Clone)]
pub struct ErrorEvent(ffi::XErrorEvent);

// SAFETY: With XInitThreads, ErrorEvent is both Send and Sync.
unsafe impl Send for ErrorEvent {}
unsafe impl Sync for ErrorEvent {}

impl ErrorEvent {
    /// Get the serial number of the failed request.
    #[allow(clippy::unnecessary_cast)]
    pub fn serial(&self) -> u64 {
        self.0.serial as u64
    }

    /// Get the error code.
    pub fn error_code(&self) -> u8 {
        self.0.error_code
    }

    /// Get the request code.
    pub fn request_code(&self) -> u8 {
        self.0.request_code
    }

    /// Get the minor opcode of the failed request.
    pub fn minor_code(&self) -> u8 {
        self.0.minor_code
    }

    /// Get the resource ID of the failed request.
    pub fn resource_id(&self) -> usize {
        self.0.resourceid as usize
    }
}

impl fmt::Debug for ErrorEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorEvent")
            .field("serial", &self.serial())
            .field("error_code", &self.error_code())
            .field("request_code", &self.request_code())
            .field("minor_code", &self.minor_code())
            .field("resource_id", &self.resource_id())
            .finish_non_exhaustive()
    }
}

/// The display pointer.
pub struct Display {
    /// The display pointer.
    ptr: NonNull<ffi::Display>,

    /// This owns the memory that the display pointer points to.
    _marker: PhantomData<Box<ffi::Display>>,
}

// SAFETY: With XInitThreads, Display is both Send and Sync.
unsafe impl Send for Display {}
unsafe impl Sync for Display {}

impl fmt::Debug for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Display").field(&self.ptr.as_ptr()).finish()
    }
}

impl Display {
    /// Open a new display.
    pub fn new(name: Option<&CStr>) -> io::Result<Self> {
        let xlib = get_xlib(&XLIB)?;

        // Make sure the error handler is registered.
        setup_error_handler(xlib);

        let name = name.map_or(std::ptr::null(), |n| n.as_ptr());
        let pointer = unsafe { xlib.open_display(name) };

        NonNull::new(pointer)
            .map(|ptr| Self {
                ptr,
                _marker: PhantomData,
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to open display"))
    }

    /// Create a new `Display` from a pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid pointer to an Xlib display. In addition, it should only be dropped if the
    /// user logically owns the display.
    pub unsafe fn from_ptr(ptr: *mut c_void) -> ManuallyDrop<Self> {
        ManuallyDrop::new(Self {
            // SAFETY: "valid" implies non-null
            ptr: NonNull::new_unchecked(ptr.cast()),
            _marker: PhantomData,
        })
    }

    /// Get the pointer to the display.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr().cast()
    }

    /// Get the default screen index for this display.
    pub fn screen_index(&self) -> usize {
        let xlib = get_xlib(&XLIB).expect("failed to load Xlib");

        // SAFETY: Valid display pointer.
        let index = unsafe { xlib.default_screen(self.ptr.as_ptr()) };

        // Cast down to usize.
        index.try_into().unwrap_or_else(|_| {
            #[cfg(feature = "tracing")]
            tracing::error!(
                "XDefaultScreen returned a value out of usize range (how?!), returning zero"
            );
            0
        })
    }
}

unsafe impl as_raw_xcb_connection::AsRawXcbConnection for Display {
    fn as_raw_xcb_connection(&self) -> *mut as_raw_xcb_connection::xcb_connection_t {
        let xlib = get_xlib(&XLIB).expect("failed to load Xlib");
        unsafe { xlib.get_xcb_connection(self.ptr.as_ptr()) }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        // SAFETY: We own the display pointer, so we can drop it.
        if let Ok(xlib) = get_xlib(&XLIB) {
            unsafe {
                xlib.close_display(self.ptr.as_ptr());
            }
        }
    }
}

/// Insert an error handler into the list.
pub fn register_error_handler(handler: ErrorHook) -> io::Result<HandlerKey> {
    // Make sure the error handler is registered.
    setup_error_handler(get_xlib(&XLIB)?);

    // Insert the handler into the list.
    let mut handlers = lock!(ERROR_HANDLERS);
    let key = handlers.insert(handler);
    Ok(HandlerKey(key))
}

/// Remove an error handler from the list.
pub fn unregister_error_handler(key: HandlerKey) {
    // Remove the handler from the list.
    let mut handlers = lock!(ERROR_HANDLERS);
    handlers.remove(key.0);
}

/// The list of error handlers.
struct HandlerList {
    /// The inner list of slots.
    slots: Vec<Slot>,

    /// The number of filled slots.
    filled: usize,

    /// The first unfilled slot.
    unfilled: usize,

    /// The last error handler hook.
    prev: ffi::XErrorHook,
}

/// A slot in the error handler list.
enum Slot {
    /// A slot that is filled.
    Filled(ErrorHook),

    /// A slot that is unfilled.
    ///
    /// This value points to the next unfilled slot.
    Unfilled(usize),
}

impl HandlerList {
    /// Create a new handler list.
    #[cfg_attr(coverage, no_coverage)]
    const fn new() -> Self {
        Self {
            slots: vec![],
            filled: 0,
            unfilled: 0,
            prev: None,
        }
    }

    /// Push a new error handler.
    ///
    /// Returns the index of the handler.
    fn insert(&mut self, handler: ErrorHook) -> usize {
        // Eat the coverage for the unreachable branch.
        #[cfg_attr(coverage, no_coverage)]
        #[inline(always)]
        fn unwrapper(slot: &Slot) -> usize {
            match slot {
                Slot::Filled(_) => unreachable!(),
                Slot::Unfilled(next) => *next,
            }
        }

        let index = self.filled;

        if self.unfilled == self.slots.len() {
            self.slots.push(Slot::Filled(handler));
            self.unfilled += 1;
        } else {
            let unfilled = self.unfilled;
            self.unfilled = unwrapper(&self.slots[unfilled]);
            self.slots[unfilled] = Slot::Filled(handler);
        }

        self.filled += 1;

        index
    }

    /// Remove an error handler.
    fn remove(&mut self, index: usize) {
        let slot = &mut self.slots[index];

        if let Slot::Filled(_) = slot {
            *slot = Slot::Unfilled(self.unfilled);
            self.unfilled = index;
            self.filled -= 1;
        }
    }

    /// Iterate over the error handlers.
    fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut ErrorHook)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Filled(handler) => Some((i, handler)),
                _ => None,
            })
    }
}

/// Static unsafe error hook slot.
struct ErrorHookSlot(Cell<Option<ffi::XErrorHook>>);

unsafe impl Sync for ErrorHookSlot {}

impl ErrorHookSlot {
    #[cfg_attr(coverage, no_coverage)]
    const fn new() -> Self {
        Self(Cell::new(None))
    }

    unsafe fn get(&self) -> Option<ffi::XErrorHook> {
        self.0.get()
    }

    #[cfg_attr(coverage, no_coverage)]
    unsafe fn set(&self, hook: ffi::XErrorHook) {
        self.0.set(Some(hook));
    }
}

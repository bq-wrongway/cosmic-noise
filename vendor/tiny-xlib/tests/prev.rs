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

// This needs to be a separate process from the others.
// This emulates another process that is using the same display.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tiny_xlib::Display;

static WAS_CALLED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn prev_handler(
    _conn: *mut x11_dl::xlib::Display,
    _event: *mut x11_dl::xlib::XErrorEvent,
) -> i32 {
    WAS_CALLED.store(true, Ordering::SeqCst);
    0
}

#[test]
fn replace_old_handler() {
    tracing_subscriber::fmt::try_init().ok();

    // Simulate setting the handler.
    unsafe {
        (x11_dl::xlib::Xlib::open().unwrap().XSetErrorHandler)(Some(prev_handler));
    }

    // Set a new handler.
    let flag = Arc::new(AtomicBool::new(false));
    tiny_xlib::register_error_handler({
        let flag = flag.clone();
        Box::new(move |_, _| {
            flag.store(true, Ordering::SeqCst);
            false
        })
    })
    .unwrap();

    // Create a display and trigger a new error.
    let display = Display::new(None).unwrap();
    trigger_error(&display);

    assert!(flag.load(Ordering::SeqCst));
    assert!(WAS_CALLED.load(Ordering::SeqCst));
}

/// Trigger an error by creating a bad drawable.
fn trigger_error(display: &Display) {
    let xlib = x11_dl::xlib::Xlib::open().unwrap();
    unsafe {
        (xlib.XCreateGC)(display.as_ptr().cast(), 0x1337, 0, std::ptr::null_mut());
        (xlib.XSync)(display.as_ptr().cast(), 0);
    }
}

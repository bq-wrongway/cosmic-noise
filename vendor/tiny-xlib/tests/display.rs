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

use as_raw_xcb_connection::AsRawXcbConnection;
use std::ffi::CString;
use std::sync::{Arc, Mutex};
use tiny_xlib::Display;

#[test]
fn smoke() {
    tracing_subscriber::fmt::try_init().ok();

    let display = Display::new(None).unwrap();
    let _conn = unsafe {
        x11rb::xcb_ffi::XCBConnection::from_raw_xcb_connection(
            display.as_raw_xcb_connection().cast(),
            false,
        )
        .unwrap()
    };
}

#[test]
fn error_handling() {
    tracing_subscriber::fmt::try_init().ok();
    let display = Display::new(None).unwrap();

    // Add a handler for the error.
    let error = Arc::new(Mutex::new(None));
    let key = tiny_xlib::register_error_handler({
        let error = error.clone();
        Box::new(move |_, event| {
            error.lock().unwrap().replace(event.clone());
            true
        })
    })
    .unwrap();

    // Create a GC with a bum window.
    trigger_error(&display);

    // The error should be set.
    tiny_xlib::unregister_error_handler(key);
    let error = error.lock().unwrap().take().unwrap();
    assert_eq!(error.error_code(), x11_dl::xlib::BadDrawable as _);
    assert_eq!(error.minor_code(), 0);

    // Eat coverage.
    let _ = format!("{:?}", error);
    let _ = format!("{:?}", key.clone());
}

#[test]
fn display_should_be_able_to_fail_creating() {
    tracing_subscriber::fmt::try_init().ok();
    let res = Display::new(Some(&CString::new("not-a-real-display").unwrap()));
    assert!(res.is_err());
}

#[test]
fn remove_and_re_insert() {
    tracing_subscriber::fmt::try_init().ok();

    let bad_flag = Arc::new(Mutex::new(false));
    let good_flag = Arc::new(Mutex::new(false));

    let key = tiny_xlib::register_error_handler({
        let bad_flag = bad_flag.clone();
        Box::new(move |_, _| {
            *bad_flag.lock().unwrap() = true;
            true
        })
    })
    .unwrap();

    // Remove the error handler.
    tiny_xlib::unregister_error_handler(key);

    // Nothing should happen.
    let display = Display::new(None).unwrap();
    trigger_error(&display);

    // The error should not be set.
    assert!(!*bad_flag.lock().unwrap());

    // Push the error handler back on with a different flag.
    let _key = tiny_xlib::register_error_handler({
        let good_flag = good_flag.clone();
        Box::new(move |_, _| {
            *good_flag.lock().unwrap() = true;
            true
        })
    })
    .unwrap();

    // Create a GC with a bum window.
    trigger_error(&display);

    // The error should be set.
    assert!(!*bad_flag.lock().unwrap());
    assert!(*good_flag.lock().unwrap());
}

/// Trigger an error by creating a bad drawable.
fn trigger_error(display: &Display) {
    tracing_subscriber::fmt::try_init().ok();

    let xlib = x11_dl::xlib::Xlib::open().unwrap();
    unsafe {
        (xlib.XCreateGC)(display.as_ptr().cast(), 0x1337, 0, std::ptr::null_mut());
        (xlib.XSync)(display.as_ptr().cast(), 0);
    }
}

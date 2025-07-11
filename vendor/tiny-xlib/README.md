# tiny-xlib

A tiny set of bindings to the [Xlib] library.

The primary contemporary library for handling [Xlib] is the [`x11-dl`] crate. However, there are three
primary issues.

1. **You should not be using Xlib in 2023.** [Xlib] is legacy code, and even that doesn't get across
    how poor the API decisions that it's locked itself into are. It has a global error hook for
    some reason, thread-safety is a mess, and it has so many soundness holes it might as well be made
    out of swiss cheese. You should not be using [Xlib]. If you *have* to use [Xlib], you should just
    run all of your logic using the much more sound [XCB] library, or, even more ideally, something
    like [`x11rb`]. Then, you take the `Display` pointer and use it for whatever legacy API you've
    locked yourself into, and use [XCB] or [`x11rb`] for everything else. Yes, I just called [GLX]
    a legacy API. It's the 2020's now. [Vulkan] and [`wgpu`] run everywhere aside from legacy machines.
    Not to mention, they support [XCB].

2. Even if you manage to use [`x11-dl`] without tripping over the legacy API, it is a massive crate.
    [Xlib] comes with quite a few functions, most of which are unnecessary in the 21st century.
    Even if you don't use any of these and just stick to [XCB], you still pay the price for it.
    Binaries that use [`x11-dl`] need to dedicate a significant amount of their binary and memory
    space to the library. Even on Release builds, I have recorded [`x11-dl`] taking up to seven
    percent of the binary.

3. Global error handling. [Xlib] has a single global error hook. This is reminiscent of the Unix
    signal handling API, in that it makes it difficult to create well-modularized programs
    since they will fight with each-other over the error handlers. However, unlike the signal
    handling API, there is no way to tell if you're replacing an existing error hook.

`tiny-xlib` aims to solve all of these problems. It provides a safe API around [Xlib] that is
conducive to being handed off to both [XCB] APIs and legacy [Xlib] APIs. The library only
imports absolutely necessary functions. In addition, it also provides a common API for
handling errors in a safe, modular way.

# Features

- Safe API around [Xlib]. See the [`Display`] structure.
- Minimal set of dependencies.
- Implements [`AsRawXcbConnection`], which allows it to be used with [XCB] APIs.
- Modular error handling.

# Non-Features

- Any API outside of opening [`Display`]s and handling errors. If this library doesn't support some
  feature, it's probably intentional. You should use [XCB] or [`x11rb`] instead. This includes:
 - Window management.
 - Any extensions outside of `Xlib-xcb`.
 - IME handling.
 - Hardware rendering.

# Examples

```no_run
use as_raw_xcb_connection::AsRawXcbConnection;
use tiny_xlib::Display;

use x11rb::connection::Connection;
use x11rb::xcb_ffi::XCBConnection;

// Open a display.
let display = Display::new(None)?;

// Get the XCB connection.
let xcb_conn = display.as_raw_xcb_connection();

// Use that pointer to create a new XCB connection.
let xcb_conn = unsafe {
    XCBConnection::from_raw_xcb_connection(xcb_conn.cast(), false)?
};

// Register a handler for X11 errors.
tiny_xlib::register_error_handler(Box::new(|_, error| {
    println!("X11 error: {:?}", error);
    false
}));

// Do whatever you want with the XCB connection.
loop {
    println!("Event: {:?}", xcb_conn.wait_for_event()?);
}
```

[Xlib]: https://en.wikipedia.org/wiki/Xlib
[XCB]: https://xcb.freedesktop.org/
[`x11-dl`]: https://crates.io/crates/x11-dl
[`x11rb`]: https://crates.io/crates/x11rb
[GLX]: https://en.wikipedia.org/wiki/GLX
[Vulkan]: https://www.khronos.org/vulkan/
[`wgpu`]: https://crates.io/crates/wgpu
[`Display`]: https://docs.rs/tiny-xlib/latest/tiny-xlib/struct.Display.html
[`AsRawXcbConnection`]: https://docs.rs/as_raw_xcb_connection/latest/as_raw_xcb_connection/trait.AsRawXcbConnection.html

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
 * Zlib license ([LICENSE-ZLIB](LICENSE-ZLIB) or https://opensource.org/licenses/Zlib)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

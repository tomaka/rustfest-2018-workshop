// Copyright 2018 Pierre Krieger
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! # Chapter 3
//!
//! The goal of this chapter is to take the code of chapter 2 and make it compile and run inside
//! of a browser!
//!
//! In order to run this code in the browser, follow these steps:
//!
//! - Install docker if you haven't done so yet.
//! - Create a docker container with the image `tomaka/rustc-emscripten`. This can be done by
//!   running `docker run --rm -it -v `pwd`:/usr/code -w /usr/code tomaka/rustc-emscripten` from
//!   the root of this repository.
//! - From inside the container, go to the `chapter-3` directory and run
//!   `cargo build --target=asmjs-unknown-emscripten`.
//! - Open the `browser.html` file included in this crate in your browser. It should automatically
//!   find the generated JavaScript code.
//!
//! In addition to `browser.html`, you are also given a file `platform.rs`. This file contains
//! platform-independant code that allows you to run an events loop and receive messages from stdin
//! in a cross-plaform way. See the usage in the `main()` function below.
//!
//! The browser doesn't support dialing to a TCP port. The only protocol that is allowed is
//! websockets. Good news, however! The `build_transport()` method in the `platform` module
//! automatically builds a transport that supports websockets. To use them, instead of dialing
//! `/ip4/1.2.3.4/tcp/1000`, you can dial `/ip4/1.2.3.4/tcp/1000/ws`.
//!
//! Additionally, please note that the browser doesn't support listening on any connection (even
//! websockets). Calling `listen_on` will trigger an error at runtime. You can use
//! `if cfg!(not(target_os = "emscripten")) { ... }` to listen only when outside of the browser.
//!
//! Good luck!

extern crate futures;
extern crate tokio_io;

#[cfg(target_os = "emscripten")]
#[macro_use]
extern crate stdweb;

mod platform;

fn main() {
    // The `PlatformSpecific` object allows you to handle the transport and stdin in a
    // cross-platform manner.
    let platform = platform::PlatformSpecific::default();

    // This builds an implementation of the `Transport` trait (similar to the `TcpConfig` object in
    // earlier chapters).
    let transport = platform.build_transport();

    // This builds a stream of messages coming from stdin.
    let stdin = platform.stdin();

    // Insert your code here!

    // Instead of `core.run()`, use `platform.run()`.
    //platform.run(final_future);
}

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

//! # Chapter 1
//!
//! Welcome to chapter 1 of this workshop!
//!
//! The code below uses libp2p, futures and tokio in order to listen for incoming connections, and
//! writes "Hello world" on any socket being opened to the server.
//!
//! Your task is to add the code (in `main.rs` as well) that dials a server and reads the message
//! being written.

extern crate futures;
extern crate libp2p;
extern crate tokio_core;
extern crate tokio_io;

use futures::{Future, Stream};
use tokio_io::io;
use tokio_core::reactor::Core;

use libp2p::Multiaddr;
use libp2p::core::Transport;

fn main() {
    // We start by building the tokio engine that will be powering the networking of
    // the application.
    let mut core = Core::new().unwrap();

    // Libp2p is using what we call *transports*. A transport is a piece of code that handles a
    // specific networking protocol. Example possible transports include: IP, TCP, UDP, DNS,
    // WebSockets, Unix sockets, WebRTC, Onion (Tor), and so on.
    //
    // Now let's build the configuration of the transports that we want to support.
    // In this example we only support TCP/IP, but libp2p supports other transport protocols and
    // you can write your owns!
    //
    // Note that we pass a handle to the tokio engine so that the networking can use it to
    // spawn sockets.
    let transport = libp2p::tcp::TcpConfig::new(core.handle());

    // Libp2p uses what is called a multiaddress to represent the address of a node, or the address
    // to listen to.
    // A multiaddress is represented through the `Multiaddr` object and can be parsed from a
    // string.
    //
    // The multiaddress `/ip4/0.0.0.0/tcp/0` corresponds to listening with any port on any address.
    let listen_multiaddr: Multiaddr = "/ip4/0.0.0.0/tcp/0"
        .parse()
        .expect("failed to parse multiaddress");

    // We ask the transport to listen on that multiaddress. We obtain `incoming_connec_stream`, a
    // stream of incoming connections. It implements the `Stream` trait of the `futures` crate.
    let (incoming_connec_stream, listened_multiaddress) = transport
        .clone()
        .listen_on(listen_multiaddr)
        .expect("multiaddress is not supported by the transport");

    // Listening also returned `listened_multiaddress`, which is a modified version of
    // `listen_multiaddr`. `listened_multiaddress` most notably contains the port that we are
    // actually listening on.
    println!("Now listening on {}", listened_multiaddress);

    // The code below assumes that you are familiar with `futures` and `tokio`!
    // We take the stream of incoming connections and apply modifiers to it in order to obtain a
    // future that represents when the stream of incoming connections is over.

    let listener_finished_future = incoming_connec_stream
        .and_then(|negotiated| {
            // For reasons outside of the scope of this chapter, each element produced by the
            // stream is in fact a future itself, so we just use `and_then` to turn a stream of
            // futures of connections into a stream of connections.
            negotiated
        })
        .for_each(|(data_stream, remote_addr)| {
            // For each incoming connection, write "Hello world" to it and return a future that
            // represents the moment when the writing finished.
            println!("Successfully received incoming connection from {}", remote_addr);
            io::write_all(data_stream, b"hello world")
                .map(|_| ())
        });
    
    // We now have `listener_finished_future`, which is a future representing the moment when
    // the listener is closed or has finished processing everything.

    // *** WORKSHOP ACTION ITEM HERE ***
    //
    // The code above listens to a port for any incoming connection and writes "Hello world" to
    // each socket that is being opened.
    //
    // Your task in this chapter is to write the dialer:
    //
    // - Parse `std::env::args().nth(1)` to retreive the address to dial.
    // - Use `transport.dial()` to dial the address. This returns a future that represents when the
    //   connection has been opened.
    //   Hint: don't forget to `unwrap()` the output of `dial()`.
    // - The future produces a tuple of a socket (of the same type as `data_stream` above) and
    //   the address of the remote.
    //   Use `tokio_io::read_to_end` to read what the socket receives and print it to stdout (hint:
    //   this should be "Hello world").
    //
    //  Tips:
    //
    // - You need to handle the situation where the user just wants to listen, and doesn't pass
    //   any address to dial. If you get stuck, you will likely need to use
    //   `futures::future::Either`, or put the futures in a `Box<Future<Item = _, Error = _>>`.
    // - You can read the documentation of libp2p by running `cargo doc --open`.
    //

    // This is a place-holder. Dial the remote and produce a future that represents the moment
    // when you've read the hello world message sent to us.
    let dialer_finished_future = futures::future::empty();

    // `final_future` is a future that contains all the behaviour that we want ; it represents the
    // moment when the both the stream of incoming connections is over and when we finished reading
    // the hello world when dialing. However nothing has actually started yet. Because we created
    // the `TcpConfig` with tokio, we need to run the future through the tokio core.
    let final_future = listener_finished_future
        .select(dialer_finished_future)
        .map(|_| ())
        .map_err(|(err, _)| err);
    core.run(final_future).unwrap();
}

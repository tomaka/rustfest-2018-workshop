// Copyright 2018 Parity Technologies (UK) Ltd.
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

extern crate futures;
extern crate libp2p_core;
extern crate libp2p_floodsub;
extern crate libp2p_identify;
extern crate libp2p_kad;
extern crate libp2p_mplex;
extern crate libp2p_peerstore;
#[cfg(not(target_os = "emscripten"))]
extern crate libp2p_tcp_transport;
extern crate libp2p_websocket;
extern crate rand;
#[cfg(not(target_os = "emscripten"))]
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_stdin;
extern crate tokio_timer;

use futures::{Future, Stream};
use self::libp2p_core::Transport;
use std::fmt::Debug;
use std::io::Error as IoError;
#[cfg(target_os = "emscripten")]
use stdweb;

#[cfg(not(target_os = "emscripten"))]
pub struct PlatformSpecific {
    core: tokio_core::reactor::Core,
}
#[cfg(target_os = "emscripten")]
pub struct PlatformSpecific {}

#[cfg(not(target_os = "emscripten"))]
impl Default for PlatformSpecific {
    fn default() -> PlatformSpecific {
        PlatformSpecific {
            core: tokio_core::reactor::Core::new().unwrap(),
        }
    }
}
#[cfg(target_os = "emscripten")]
impl Default for PlatformSpecific {
    fn default() -> PlatformSpecific {
        PlatformSpecific {}
    }
}

#[cfg(not(target_os = "emscripten"))]
impl PlatformSpecific {
    pub fn build_transport(
        &self,
    ) -> libp2p_core::transport::OrTransport<
        libp2p_websocket::WsConfig<libp2p_tcp_transport::TcpConfig>,
        libp2p_tcp_transport::TcpConfig,
    > {
        let tcp = libp2p_tcp_transport::TcpConfig::new(self.core.handle());
        libp2p_websocket::WsConfig::new(tcp.clone()).or_transport(tcp)
    }

    pub fn stdin(&self) -> impl Stream<Item = String, Error = IoError> {
        use std::mem;

        let mut buffer = Vec::new();
        tokio_stdin::spawn_stdin_stream_unbounded()
            .map_err(|_| -> IoError { panic!() })
            .filter_map(move |msg| {
                if msg != b'\r' && msg != b'\n' {
                    buffer.push(msg);
                    return None;
                } else if buffer.is_empty() {
                    return None;
                }

                Some(String::from_utf8(mem::replace(&mut buffer, Vec::new())).unwrap())
            })
    }

    pub fn run<F>(mut self, future: F)
    where
        F: Future,
        F::Error: Debug,
    {
        self.core.run(future).unwrap();
    }
}
#[cfg(target_os = "emscripten")]
impl PlatformSpecific {
    pub fn build_transport(&self) -> libp2p_websocket::BrowserWsConfig {
        stdweb::initialize();
        libp2p_websocket::BrowserWsConfig::new()
    }

    pub fn stdin(&self) -> impl Stream<Item = String, Error = IoError> {
        use futures::sync::mpsc;
        let (tx, rx) = mpsc::unbounded();

        let cb = move |txt: String| {
            let _ = tx.unbounded_send(txt);
        };

        js! {
            var cb = @{cb};
            document.getElementById("stdin_form")
                .addEventListener("submit", function(event) {
                    var elem = document.getElementById("stdin");
                    var txt = elem.value;
                    elem.value = "";
                    cb(txt);
                    event.preventDefault();
                });
        };

        rx.map_err(|_| -> IoError { unreachable!() })
    }

    pub fn run<F>(self, future: F)
    where
        F: Future + 'static,
        F::Item: Debug,
        F::Error: Debug,
    {
        use futures::{executor, Async};
        use std::sync::{Arc, Mutex};

        let future_task = executor::spawn(future);

        struct Notifier<T> {
            me: Mutex<Option<executor::NotifyHandle>>,
            task: Arc<Mutex<executor::Spawn<T>>>,
        }
        let notifier = Arc::new(Notifier {
            me: Mutex::new(None),
            task: Arc::new(Mutex::new(future_task)),
        });

        let notify_handle = executor::NotifyHandle::from(notifier.clone());
        *notifier.me.lock().unwrap() = Some(notify_handle.clone());

        notify_handle.notify(0);
        stdweb::event_loop();

        unsafe impl<T> Send for Notifier<T> {}
        unsafe impl<T> Sync for Notifier<T> {}
        impl<T> executor::Notify for Notifier<T>
        where
            T: Future,
            T::Item: Debug,
            T::Error: Debug,
        {
            fn notify(&self, _: usize) {
                let task = self.task.clone();
                let me = self.me.lock().unwrap().as_ref().unwrap().clone();
                stdweb::web::set_timeout(
                    move || {
                        let val = task.lock().unwrap().poll_future_notify(&me, 0);
                        match val {
                            Ok(Async::Ready(item)) => println!("finished: {:?}", item),
                            Ok(Async::NotReady) => (),
                            Err(err) => panic!("error: {:?}", err),
                        }
                    },
                    0,
                );
            }
        }
    }
}

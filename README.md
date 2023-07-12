# async-event

An efficient async condition variable for lock-free algorithms, a.k.a.
"eventcount".

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/asynchronics/async-event#license)


## Overview

[Eventcount][eventcount]-like primitives are useful to make some operations on a
lock-free structure blocking, for instance to transform bounded queues into
bounded channels. Such a primitive allows an interested task to block until a
predicate is satisfied by checking the predicate each time it receives a
notification.

While functionally similar to the [event_listener] crate, this implementation is
specialized for the `async` case and tries to be more efficient by limiting the
number of locking operations on the mutex-protected list of notifiers: the lock
is typically taken only once for each time a waiter is blocked and once for
notifying, thus reducing the need for synchronization operations. Finally,
spurious wake-ups are only generated in very rare circumstances.

This library is an offshoot of [Asynchronix][asynchronix], an ongoing effort at
a high performance asynchronous computation framework for system simulation.

[event_listener]: https://docs.rs/event_listener/latest/event_listener/
[eventcount]: https://www.1024cores.net/home/lock-free-algorithms/eventcounts
[asynchronix]: https://github.com/asynchronics/asynchronix

## Usage

This crate needs more testing and hasn't been released to crates.io yet. Use at
your own risk by adding this to your `Cargo.toml`:

```toml
[dependencies]
async-event = { git = "https://github.com/asynchronics/async-event.git" }
```

## Example

A multi-producer, multi-consumer channel of capacity 1 for sending
`NonZeroUsize` values:

```rust
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_event::Event;

// Data stored by the channel.
struct Inner {
    sender_notifier: Event,
    receiver_notifier: Event,
    value: AtomicUsize,
}

// The sending side of the channel.
#[derive(Clone)]
struct Sender {
    inner: Arc<Inner>,
}

// The receiving side of the channel.
#[derive(Clone)]
struct Receiver {
    inner: Arc<Inner>,
}

// Creates an empty channel.
fn channel() -> (Sender, Receiver) {
    let inner = Arc::new(Inner {
        sender_notifier: Event::new(),
        receiver_notifier: Event::new(),
        value: AtomicUsize::new(0),
    });
    (
        Sender {
            inner: inner.clone(),
        },
        Receiver { inner },
    )
}

impl Sender {
    // Sends a value asynchronously.
    async fn send(&self, value: NonZeroUsize) {
        // Wait until the predicate returns `Some`, i.e. until the atomic value
        // is found to be zero (empty channel) and the new value is set.
        self.inner
            .sender_notifier
            .wait_until(|| {
                self.inner
                    .value
                    .compare_exchange(0, value.get(), Ordering::Relaxed, Ordering::Relaxed)
                    .ok()
            })
            .await;

        // Let one of the blocked receivers (if any) know that a value is
        // available.
        self.inner.receiver_notifier.notify(1);
    }
}

impl Receiver {
    // Receives a value asynchronously.
    async fn recv(&self) -> NonZeroUsize {
        // Wait until the predicate returns `Some(value)`, i.e. when the atomic
        // value becomes non-zero (the channel contains an actual value).
        let value = self
            .inner
            .receiver_notifier
            .wait_until(|| NonZeroUsize::new(self.inner.value.swap(0, Ordering::Relaxed)))
            .await;

        // Let one of the blocked senders (if any) know that the value slot is
        // empty.
        self.inner.sender_notifier.notify(1);

        value
    }
}
```

## Safety

This is a low-level primitive and as such its implementation relies on `unsafe`.
The test suite makes extensive use of [Loom] to assess its correctness. As
amazing as it is, however, Loom is only a tool: it cannot formally prove the
absence of data races.

[Loom]: https://github.com/tokio-rs/loom


## License

This software is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT), at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

# async-event

An efficient async condition variable for lock-free algorithms, a.k.a.
"eventcount".

[![Cargo](https://img.shields.io/crates/v/async-event.svg)](https://crates.io/crates/async-event)
[![Documentation](https://docs.rs/async-event/badge.svg)](https://docs.rs/async-event)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/asynchronics/async-event#license)

## Overview

[Eventcount][eventcount]-like primitives are useful to make some operations on a
lock-free structure blocking, for instance to transform bounded queues into
bounded channels. Such a primitive allows an interested task to block until a
predicate is satisfied by checking the predicate each time it receives a
notification.

While functionally similar to the [event_listener] crate, this implementation is
more opinionated and limited to the `async` case. It strives to be more
efficient, however, by limiting the amount of locking operations on the
mutex-protected list of notifiers: the lock is typically taken only once for
each time a waiter is blocked and once for notifying, thus reducing the need for
synchronization operations. Finally, spurious wake-ups are only generated in
very rare circumstances.

Note that if you only need to send notifications to a single task, you may use
instead the [Diatomic Waker][diatomic-waker] crate for extra performance.

This library is an offshoot of [Asynchronix][asynchronix], an ongoing effort at
a high performance asynchronous computation framework for system simulation. It
is also used in the [Tachyonix][tachyonix] MPSC channel.

[event_listener]: https://docs.rs/event_listener/latest/event_listener/
[eventcount]: https://www.1024cores.net/home/lock-free-algorithms/eventcounts
[diatomic-waker]: https://github.com/asynchronics/diatomic-waker
[asynchronix]: https://github.com/asynchronics/asynchronix
[tachyonix]: https://github.com/asynchronics/tachyonix

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
async-event = "0.2.0"
```

## Differences with `event_listener`

This `Event` primitive is expected to be faster than that of the
`event_listener` crate in the general case. That being said, your mileage may
vary depending on your particular application and you should probably benchmark
both.

The API is more opinionated and designed to preventing potential misuse such as:

- *Forgetting to check again the predicate after requesting a notification, i.e.
  after a call to `Event::listen()` in the `event_listener` crate*.
  `async-event` provides instead the `Event::wait_until` method which takes care
  of checking the predicate whenever necessary to prevent races.
- *Confusion between `notify` and `notify_additional` in the `event_listener`
  crate*. Our experience and the API of other similar libraries suggest that the
  latter is almost always what the user needs, so the `notify*` methods in this
  crate actually behave like `notify_additional` in the `event_listener` crate.
- *Inadequate atomic synchronization of the predicate*. The `notify*` and
  `wait_until` methods always insert atomic fences to ensure proper
  synchronization: there is no equivalent to `notify_additional_relaxed`.


## Examples

### Send a non-zero value asynchronously

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use futures_executor::block_on;

use async_event::Event;

let value = Arc::new(AtomicUsize::new(0));
let event = Arc::new(Event::new());

// Set a non-zero value concurrently.
thread::spawn({
    let value = value.clone();
    let event = event.clone();

    move || {
        // A relaxed store is sufficient here: `Event::notify*` methods insert
        // atomic fences to warrant adequate synchronization.
        value.store(42, Ordering::Relaxed);
        event.notify_one();
    }
});

// Wait until the value is set.
block_on(async move {
    let v = event
        .wait_until(|| {
            // A relaxed load is sufficient here: `Event::wait_until` inserts
            // atomic fences to warrant adequate synchronization.
            let v = value.load(Ordering::Relaxed);
            if v != 0 { Some(v) } else { None }
        })
        .await;

     assert_eq!(v, 42);
});
```

### Single-slot MPMC channel for non-zero values

See [implementation](examples/mpmc_channel.rs) in the `examples` directory.

## Safety

This is a low-level primitive and as such its implementation relies on `unsafe`.
The test suite makes extensive use of [Loom] and MIRI to assess its correctness.
As amazing as they are, however, Loom and MIRI cannot formally prove the absence
of data races so soundness issues _are_ possible.

[Loom]: https://github.com/tokio-rs/loom


## License

This software is licensed under the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT), at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

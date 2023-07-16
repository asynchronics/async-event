//! A multi-producer, multi-consumer channel of capacity 1 for sending
//! `NonZeroUsize` values.

use async_event::Event;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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

#[tokio::main]
async fn main() {
    let (s1, r1) = channel();
    let s2 = s1.clone();
    let r2 = r1.clone();

    // Receivers.
    let task1 = tokio::spawn(async move { r1.recv().await });
    let task2 = tokio::spawn(async move { r2.recv().await });

    // Senders.
    tokio::spawn(async move {
        s1.send(NonZeroUsize::new(1).unwrap()).await;
    });
    tokio::spawn(async move {
        s2.send(NonZeroUsize::new(2).unwrap()).await;
    });

    println!("Task 1 received value {}", task1.await.unwrap());
    println!("Task 2 received value {}", task2.await.unwrap());
}

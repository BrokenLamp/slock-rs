use futures::executor::block_on;
use lazy_static::lazy_static;
use slock::*;
use std::sync::atomic::{AtomicU8, Ordering};

#[test]
fn synchronous() {
    let lock = Slock::new(5);
    block_on(lock.set(|v| v + 1));
    let new_value = block_on(lock.get_clone());
    assert_eq!(new_value, 6);
}

/// Many mutations should be able to happen at the same time without loss.
#[test]
fn asynchronous() {
    let lock = Slock::new(0i32);

    block_on(async {
        let f_1 = lock.set(|v| v + 1);
        let f_2 = lock.set(|v| v + 1);
        let f_3 = lock.set(|v| v + 1);
        let f_4 = lock.set(|v| v + 1);
        let f_5 = lock.set(|v| v + 1);
        futures::join!(f_1, f_2, f_3, f_4, f_5);
    });

    assert_eq!(block_on(lock.get_clone()), 5);
}

/// Should be able to create multiple references to the same lock.
#[test]
fn reference_counting() {
    let lock_1 = Slock::new(0);
    let lock_2 = lock_1.split();
    block_on(lock_1.set(|_| 1));
    assert_eq!(block_on(lock_1.get_clone()), 1);
    assert_eq!(block_on(lock_2.get_clone()), 1);
}

#[test]
fn mapping() {
    struct User {
        name: &'static str,
        age: i32,
    };

    let lock = Slock::new(User {
        name: "Bob",
        age: 32,
    });

    let name = block_on(lock.map(|v| v.name));
    let age = block_on(lock.map(|v| v.age));

    assert_eq!(name, "Bob");
    assert_eq!(age, 32);
}

/// A slock containing a vector should be able to asynchronously push.
#[test]
fn vector() {
    let vec: Vec<i32> = Vec::new();
    let lock = Slock::new(vec);
    block_on(lock.push(1));
    assert_eq!(block_on(lock.map(|v| v[0])), 1);
}

/// Old value should Drop when a new value is created.
#[test]
fn destruction() {
    lazy_static! {
        static ref COUNT: AtomicU8 = AtomicU8::new(0);
    }

    struct Struct;

    impl Drop for Struct {
        fn drop(&mut self) {
            COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let lock = Slock::new(Struct);
    block_on(lock.set(|_| Struct));
    block_on(lock.set(|_| Struct));
    std::mem::drop(lock);
    assert_eq!(COUNT.load(Ordering::SeqCst), 3);
}

/// Old value should not Drop when returned back to the lock.
#[test]
fn non_destruction() {
    lazy_static! {
        static ref COUNT: AtomicU8 = AtomicU8::new(0);
    }

    struct Struct;

    impl Drop for Struct {
        fn drop(&mut self) {
            COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    let lock = Slock::new(Struct);
    block_on(lock.set(|v| v));
    block_on(lock.set(|v| v));
    std::mem::drop(lock);
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn arrithmetic() {
    let mut lock = Slock::new(0i32);
    lock += 1;
    assert_eq!(block_on(lock.get()), 1);
    lock -= 1;
    assert_eq!(block_on(lock.get()), 0);
    lock += 1;
    lock *= 2;
    assert_eq!(block_on(lock.get()), 2);
    lock /= 2;
    assert_eq!(block_on(lock.get()), 1);
    lock %= 1;
    assert_eq!(block_on(lock.get()), 0);
}

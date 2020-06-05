#![cfg(feature = "blocking")]

use futures::executor::block_on;
use slock::*;

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

#[test]
fn cloning() {
    let lock = Slock::new(String::from("hello"));
    let lock_cloned = lock.clone();
    block_on(lock.set(|_| String::from("goodbye")));
    assert_eq!(block_on(lock_cloned.get_clone()), "hello");
}

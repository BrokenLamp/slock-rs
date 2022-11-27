use lazy_static::lazy_static;
use slock::*;
use std::sync::atomic::{AtomicU8, Ordering};

#[tokio::test]
async fn synchronous() {
    let lock = Slock::new(5);
    lock.set(|v| v + 1).await;
    let new_value = lock.get_clone().await;
    assert_eq!(new_value, 6);
}

/// Many mutations should be able to happen at the same time without loss.
#[tokio::test]
async fn asynchronous() {
    let lock = Slock::new(0i32);

    async {
        let f_1 = lock.set(|v| v + 1);
        let f_2 = lock.set(|v| v + 1);
        let f_3 = lock.set(|v| v + 1);
        let f_4 = lock.set(|v| v + 1);
        let f_5 = lock.set(|v| v + 1);
        tokio::join!(f_1, f_2, f_3, f_4, f_5);
    }
    .await;

    assert_eq!(lock.get().await, 5);
}

/// Should be able to create multiple references to the same lock.
#[tokio::test]
async fn reference_counting() {
    let lock_1 = Slock::new(0);
    let lock_2 = lock_1.clone();
    lock_1.set(|_| 1).await;
    assert_eq!(lock_1.get().await, 1);
    assert_eq!(lock_2.get().await, 1);
}

#[tokio::test]
async fn mapping() {
    struct User {
        name: &'static str,
        age: i32,
    }

    let lock = Slock::new(User {
        name: "Bob",
        age: 32,
    });

    let name = lock.map(|v| v.name).await.unwrap();
    let age = lock.map(|v| v.age).await.unwrap();

    assert_eq!(name, "Bob");
    assert_eq!(age, 32);
}

/// A slock containing a vector should be able to asynchronously push.
#[tokio::test]
async fn vector() {
    let vec: Vec<i32> = Vec::new();
    let lock = Slock::new(vec);
    lock.push(1).await;
    assert_eq!(lock.map(|v| v[0]).await.unwrap(), 1);
}

/// Old value should Drop when a new value is created.
#[tokio::test]
async fn destruction() {
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
    lock.set(|_| Struct).await;
    lock.set(|_| Struct).await;
    std::mem::drop(lock);
    assert_eq!(COUNT.load(Ordering::SeqCst), 3);
}

/// Old value should not Drop when returned back to the lock.
#[tokio::test]
async fn non_destruction() {
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
    lock.set(|v| v).await;
    lock.set(|v| v).await;
    std::mem::drop(lock);
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
}

use futures::executor::block_on;
use slock::*;

#[test]
fn synchronous() {
    let lock = Slock::new(5);
    block_on(lock.set(|v| v + 1));
    let new_value = block_on(lock.get_clone());
    assert_eq!(new_value, 6);
}

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

#[test]
fn reference_counting() {
    let lock_1 = Slock::new(0);
    let lock_2 = lock_1.split();
    block_on(lock_1.set(|_| 1));
    assert_eq!(block_on(lock_1.get_clone()), 1);
    assert_eq!(block_on(lock_2.get_clone()), 1);
}

#[test]
fn lock_function() {
    let lock = lock(0);
    assert_eq!(block_on(lock.get_clone()), 0);
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

#[test]
fn vector() {
    let vec: Vec<i32> = Vec::new();
    let lock = Slock::new(vec);
    block_on(lock.push(1));
    assert_eq!(block_on(lock.map(|v| v[0])), 1);
}

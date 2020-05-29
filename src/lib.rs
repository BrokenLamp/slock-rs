//! A Slock, or Smart Lock, is a smart wrapper around an atomically reference counted read/write lock.
//!
//! All accesses and modifications are done in a contained manner.
//! This ensures that threads will never deadlock on a Slock operation.
//!
//! ```rust
//! use slock::*;
//!
//! async {
//!     // Create a new lock with an initial value
//!     let lock = Slock::new(5i32);
//!
//!     // Change the lock's value
//!     lock.set(|v| v + 1).await;
//!
//!     // Get the lock's value
//!     let value = lock.get().await;
//!     println!("{}", value); // 6
//!
//!     // Use in multiple threads
//!     //futures::join!(
//!     //    do_something_in_a_thread(lock.clone()),
//!     //    do_something_else_in_a_thread(lock.clone()),
//!     //    do_another_thing_in_a_thread(lock.clone()),
//!     //);
//! };
//! ```
//!
//! ## Things not to do
//!
//! ### Don't access a Slock from within another
//!
//! Bad:
//! ```rust
//! use slock::*;
//! use futures::executor::block_on;
//!
//! async {
//!     let lock_1 = Slock::new(0i32);
//!     let lock_2 = Slock::new(1i32);
//!
//!     // Add the value of lock_2 to lock_2
//!     lock_1.set(|v| {
//!         v + block_on(lock_2.get())
//!     });
//! };
//! ```
//!
//! Good:
//! ```rust
//! use slock::*;
//!
//! async {
//!     let lock_1 = Slock::new(0i32);
//!     let lock_2 = Slock::new(1i32);
//!
//!     // Add the value of lock_2 to lock_1
//!     let v_2 = lock_2.get().await;
//!     lock_1.set(|v| v + v_2);
//! };
//! ```

use std::sync::{Arc, RwLock};

pub struct Slock<T> {
    lock: Arc<RwLock<T>>,
}

impl<T> Slock<T> {
    /// Create a new Slock with a given initial value.
    pub fn new(value: T) -> Self {
        Self {
            lock: Arc::new(RwLock::new(value)),
        }
    }

    /// Extract inner values from within a Slock
    /// ```rust
    /// use slock::*;
    /// let lock = Slock::new((0, 1, 2));
    /// let name = lock.map(|v| v.1);
    /// ```
    pub async fn map<F, U>(&self, mapper: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        match self.lock.read() {
            Ok(v) => mapper(&*v),
            Err(_) => panic!("Slock could not read for map!"),
        }
    }

    pub async fn set<F>(&self, setter: F)
    where
        F: FnOnce(T) -> T,
    {
        match self.lock.write() {
            Ok(mut v) => {
                let ptr = &mut *v as *mut T;
                unsafe {
                    let new = setter(ptr.read());
                    ptr.write(new);
                }
            }
            Err(_) => panic!("Slock could not write!"),
        }
    }

    /// A setter using a mutable reference
    pub async fn set_ref<F>(&self, setter: F)
    where
        F: FnOnce(&T) -> T,
    {
        match self.lock.write() {
            Ok(mut v) => {
                setter(&mut v);
            }
            Err(_) => panic!("Slock could not write!"),
        }
    }

    pub fn split(&self) -> Self {
        Self {
            lock: self.lock.clone(),
        }
    }

    pub unsafe fn get_arc(&self) -> Arc<RwLock<T>> {
        self.lock.clone()
    }
}

impl<T: Clone> Slock<T> {
    pub async fn get_clone(&self) -> T {
        match self.lock.read() {
            Ok(v) => v.clone(),
            Err(_) => panic!("Slock could not read for clone!"),
        }
    }
}

impl<T: Copy> Slock<T> {
    pub async fn get(&self) -> T {
        match self.lock.read() {
            Ok(v) => *v,
            Err(_) => panic!("Slock could not read for clone!"),
        }
    }
}

impl<T> Slock<Vec<T>> {
    pub async fn push(&self, value: T) {
        self.set(|mut v| {
            v.push(value);
            v
        })
        .await;
    }
}

pub fn lock<T>(value: T) -> Slock<T> {
    Slock::new(value)
}

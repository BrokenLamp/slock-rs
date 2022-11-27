//! A [`Slock`](struct.Slock.html), or Smart Lock, is a smart wrapper around an atomically reference counted read/write lock.
//!
//! All accesses and modifications are contained, ensuring that threads will never deadlock on a Slock operation.
//!
//! ```rust
//! use slock::*;
//!
//! # async fn do_something_in_a_thread(_: Slock<i32>) {}
//! # async fn do_something_else_in_a_thread(_: Slock<i32>) {}
//! # async fn do_another_thing_in_a_thread(_: Slock<i32>) {}
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
//!     tokio::join!(
//!         do_something_in_a_thread(lock.split()),
//!         do_something_else_in_a_thread(lock.split()),
//!         do_another_thing_in_a_thread(lock.split()),
//!     );
//! };
//! ```
//!
//! ## Things not to do
//!
//! ### Don't access a Slock from within another
//!
//! Bad:
//! ```rust,ignore
//! # use slock::*;
//! # use futures::executor::block_on;
//! # async {
//! let lock_1 = Slock::new(0i32);
//! let lock_2 = Slock::new(1i32);
//!
//! // Add the value of lock_2 to lock_1
//! lock_1.set(|v| v + block_on(lock_2.get())).await;
//! # };
//! ```
//!
//! Good:
//! ```rust
//! # use slock::*;
//! # async {
//! let lock_1 = Slock::new(0i32);
//! let lock_2 = Slock::new(1i32);
//!
//! // Add the value of lock_2 to lock_1
//! let v_2 = lock_2.get().await;
//! lock_1.set(|v| v + v_2).await;
//! # };
//! ```

use std::{cmp::Eq, collections::HashMap, hash::Hash, sync::Arc};

use tokio::{
    sync::RwLock,
    time::{error::Elapsed, timeout},
};

pub struct SlockData<T> {
    pub version: u32,
    pub value: T,
    pub hook: Option<Box<dyn FnMut(&T)>>,
}

pub struct Slock<T> {
    lock: Arc<RwLock<SlockData<T>>>,
}

impl<T> Slock<T> {
    /// Create a new Slock with a given initial value.
    pub fn new(value: T) -> Self {
        let data = SlockData {
            version: 0,
            value,
            hook: None,
        };
        Self {
            lock: Arc::new(RwLock::new(data)),
        }
    }

    /// Extract inner values from within a Slock
    /// ```rust
    /// # use slock::*;
    /// # struct User { name: &'static str };
    /// # let lock = Slock::new(User {name: "bobs"});
    /// # async {
    /// let name = lock.map(|v| v.name).await;
    /// # };
    /// ```
    pub async fn map<F, U>(&self, mapper: F) -> Result<U, Elapsed>
    where
        F: FnOnce(&T) -> U,
    {
        let v = self.lock.read().await;
        timeout(std::time::Duration::from_secs(1), async {
            mapper(&v.value)
        })
        .await
    }

    /// A setter for changing the internal data of the lock.
    /// ```rust
    /// # use slock::*;
    /// let lock = Slock::new(1i32);
    ///
    /// # async {
    /// lock.set(|v| v + 1).await;
    /// lock.set(|_| 6).await;
    /// # };
    /// ```
    pub async fn set<F>(&self, setter: F)
    where
        F: FnOnce(T) -> T,
    {
        let mut data = self.lock.write().await;
        let ptr = &mut data.value as *mut T;
        let new = timeout(std::time::Duration::from_secs(1), async {
            setter(unsafe { ptr.read() })
        })
        .await;
        if let Ok(new) = new {
            timeout(std::time::Duration::from_secs(1), async {
                data.hook.as_mut().map(|hook| hook(&new));
            })
            .await
            .ok();
            unsafe { ptr.write(new) };
        }

        data.version += 1;
    }

    /// Create's a new lock pointing to the same data.
    /// Modifying the data in the new lock will result in
    /// seeing the same change in the old lock.
    /// ```
    /// # use slock::*;
    /// let lock = Slock::new(0i32);
    /// let the_same_lock = lock.split();
    /// ```
    #[deprecated = "Use `clone()` instead"]
    pub fn split(&self) -> Self {
        Self {
            lock: self.lock.clone(),
        }
    }

    /// Returns the lock's atomic reference counter.
    /// This is unsafe as using it can no longer guarantee
    /// deadlocks won't occur.
    pub unsafe fn get_raw_arc(&self) -> Arc<RwLock<SlockData<T>>> {
        self.lock.clone()
    }

    pub async fn hook<F: 'static>(&self, hook: F)
    where
        F: FnMut(&T),
    {
        let mut data = self.lock.write().await;
        data.hook = Some(Box::new(hook));
    }
}

impl<T> Clone for Slock<T> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
        }
    }
}

impl<T: Clone> Slock<T> {
    /// Returns a clone of the lock's data.
    pub async fn get_clone(&self) -> T {
        let data = self.lock.read().await;
        data.value.clone()
    }

    /// Create a new lock with data clone from this one.
    pub async fn clone_deep(&self) -> Self {
        return Slock::new(self.get_clone().await);
    }
}

impl<T> Slock<Vec<T>> {
    /// Asyncronously push to a vec.
    /// Note that due to the nature of async code, order cannot be guaranteed.
    pub async fn push(&self, value: T) {
        self.set(|mut v| {
            v.push(value);
            v
        })
        .await;
    }
}

impl<T> Slock<Slock<T>> {
    /// Converts from `Slock<Slock<T>>` to `Slock<T>`
    pub async fn flatten(&self) -> Slock<T> {
        self.map(|inner| inner.clone()).await.unwrap()
    }
}

/// ## HashMaps
///
/// Slock has built-in convenience methods for working with `Slock<HashMap<Slock>>`s
pub type SlockMap<K, V> = Slock<HashMap<K, Slock<V>>>;

impl<K: Eq + Hash + Copy, V> SlockMap<K, V> {
    pub fn new_map() -> Slock<HashMap<K, Slock<V>>> {
        let map: HashMap<K, Slock<V>> = HashMap::new();
        Slock::new(map)
    }

    pub async fn insert<F>(&self, key: K, setter: F)
    where
        F: FnOnce(Option<V>) -> V,
    {
        if let Some(data) = self.from_key(key).await {
            data.set(|v| setter(Some(v))).await;
        } else {
            self.set(|mut hash_map| {
                hash_map.insert(key, Slock::new(setter(None)));
                hash_map
            })
            .await;
        }
    }

    pub async fn from_key(&self, key: K) -> Option<Slock<V>> {
        self.map(|hash_map| {
            let key = key;
            hash_map.get(&key).map(|inner| inner.clone())
        })
        .await
        .unwrap()
    }
}

impl<T: Copy> Slock<T> {
    /// If a lock's data implements copy, this will return an owned copy of it.
    pub async fn get(&self) -> T {
        let data = self.lock.read().await;
        data.value
    }
}

pub mod blocking;

unsafe impl<T: Send> Send for Slock<T> {}
unsafe impl<T: Send> Sync for Slock<T> {}

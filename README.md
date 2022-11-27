# Slock

A mutex for Rust that never deadlocks.

A Slock, or Smart Lock, is a smart wrapper around an atomically reference counted read/write lock.

All accesses and modifications are done in a contained manner.
This ensures that threads will never deadlock on a Slock operation.

```rust
// Create a new lock with an initial value
let lock = Slock::new(5i32);

// Change the lock's value
lock.set(|v| v + 1).await;

// Get the lock's value if Copy
let value = lock.get().await;
// Get the lock's value if Clone
let value = lock.get_clone().await;

assert_eq!(value, 6);
```

It's also possible to extract only the data you need from larger structures without the need to clone the entire thing.

```rust
struct User {
    name: String,
    age: i32,
}

let user = Slock::new(User {
    name: "Bob",
    age: 32,
});

// Extract something that is Copy
let age = user.map(|v| v.age).await;

// Extract something that is Clone
let name = user.map(|v| v.name.clone()).await;

// Increment `age` by 1
user.set(|v| v.age += 1).await;
```

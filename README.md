# smartlock-rs

A mutex for Rust that never deadlocks.

A Slock, or Smart Lock, is a smart wrapper around an atomically reference counted read/write lock.

All accesses and modifications are done in a contained manner.
This ensures that threads will never deadlock on a Slock operation.

```rust
// Create a new lock with an initial value
let lock = Slock::new(5i32);

// Change the lock's value
lock.set(|v| v + 1).await;

// Get the lock's value
let value = lock.get().await;
// Or if the value doesn't implement copy
let value = lock.get_clone().await;

assert_eq!(value, 6);
```

It's also possible to extract only the data you need from larger structures without the need to clone the entire thing.

```rust
// A user struct that doesn't implement copy
struct User {
    name: String,
    age: i32,
    // ... lots of other things
}

let user = Slock::new(User {
    name: "Bob",
    age: 32,
    // ... lots of other things
});

// Get just the name
// This performs a clone on only the name
let name = user.map(|v| v.name.clone()).await;

// Get just the age
// Extracts only the age, leaving everything else untouched
let age = user.map(|v| v.age).await;
```

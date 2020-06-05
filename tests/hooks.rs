use futures::executor::block_on;
use slock::*;

#[test]
fn basic_hooks() {
    unsafe {
        let lock = Slock::new(());
        static mut COUNT: i32 = 0;
        lock.hook(|_| COUNT += 1);
        block_on(lock.set(|_| ()));
        block_on(lock.set(|_| ()));
        block_on(lock.set(|_| ()));
        assert_eq!(COUNT, 3);
    }
}

use slock::*;

#[tokio::test]
async fn basic_hooks() {
    // SAFETY: Required to increment the static counter
    unsafe {
        let lock = Slock::new(());
        static mut COUNT: i32 = 0;
        lock.hook(|_| COUNT += 1).await;
        lock.set(|_| ()).await;
        lock.set(|_| ()).await;
        lock.set(|_| ()).await;
        assert_eq!(COUNT, 3);
    }
}

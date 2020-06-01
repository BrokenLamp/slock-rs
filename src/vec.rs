use super::*;

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

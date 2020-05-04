use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SmartLock<T: Clone> {
    lock: Arc<RwLock<T>>,
}

impl<T: Clone> SmartLock<T> {
    pub fn new(value: T) -> Self {
        SmartLock {
            lock: Arc::new(RwLock::new(value)),
        }
    }

    pub async fn get_clone(&self) -> T {
        match self.lock.read() {
            Ok(v) => v.clone(),
            Err(_) => panic!("SmartLock could not read for clone!"),
        }
    }

    pub async fn map<F, U>(&self, mapper: F) -> U
    where
        F: FnOnce(&T) -> U,
    {
        match self.lock.read() {
            Ok(v) => mapper(&*v),
            Err(_) => panic!("SmartLock could not read for map!"),
        }
    }

    pub async fn set<F>(&self, setter: F)
    where
        F: FnOnce(T) -> T,
    {
        match self.lock.write() {
            Ok(mut v) => {
                *v = setter(v.clone());
            }
            Err(_) => panic!("SmartLock could not write!"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

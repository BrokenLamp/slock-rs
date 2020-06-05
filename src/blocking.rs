#![cfg(feature = "blocking")]

use crate::*;
use futures::executor::block_on;

macro_rules! impl_op {
    ($name:ident, $name_assign:ident, $fun:ident, $op:tt) => {
        impl<T> std::ops::$name_assign<T> for Slock<T>
        where
            T: Copy + std::ops::$name<T, Output = T>,
            T: From<T>,
        {
            fn $fun(&mut self, other: T) {
                block_on(self.set(|v| v $op other));
            }
        }
    };
}

impl_op!(Add, AddAssign, add_assign, +);
impl_op!(Sub, SubAssign, sub_assign, -);
impl_op!(Mul, MulAssign, mul_assign, *);
impl_op!(Div, DivAssign, div_assign, /);
impl_op!(Rem, RemAssign, rem_assign, %);

impl<T: Clone> Clone for Slock<T> {
    /// Creates a clone of the lock and its data.
    /// This operation is blocking.
    /// Prefer `clone_async`
    fn clone(&self) -> Self {
        return Slock::new(block_on(self.get_clone()));
    }
}

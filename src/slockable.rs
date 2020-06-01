use super::Slock;

pub trait Slockable
where
    Self: std::marker::Sized,
{
    type Slocker;

    fn get_slocker(slock: Slock<Self>) -> Self::Slocker;
}

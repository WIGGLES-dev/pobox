use crate::concurrency::Notify;

/// users can't actually construct this type safely
pub(crate) enum LockState {
    Locked,
    Unlocked,
}

pub struct BorrowBuilder {}
unsafe impl ActorBorrowable for BorrowBuilder {}

pub unsafe trait ActorBorrowable {}

pub struct ActorBorrow<'a, T> {
    borrow: &'a T,
    unlock: Notify,
}

pub struct ActorBorrowMut<'a, T> {
    borrow: &'a mut T,
    unlock: Notify,
}

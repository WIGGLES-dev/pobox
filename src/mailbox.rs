use std::{
    ops::{Coroutine, CoroutineState},
    pin::Pin,
};

pub trait Mailbox<'a, T> {
    type Output;
    fn try_send(&'a mut self, msg: T) -> Self::Output;
}

pub trait OwnedMailbox<T>: Sized {
    fn to_owned(self) -> Self;
}

pub trait AsyncMailbox<'a, T>: Mailbox<'a, T> {
    type Output;
    fn send(&'a mut self, msg: T) -> <Self as AsyncMailbox<'a, T>>::Output;
}

pub trait BlockingMailbox<'a, T>: Mailbox<'a, T> {
    type Output;
    fn send_blocking(&'a mut self, msg: T) -> <Self as BlockingMailbox<'a, T>>::Output;
}

pub trait StackMailbox<'a, T>: Mailbox<'a, T> {
    type Output;
    fn send_ref(&'a mut self, msg: &'a T) -> <Self as StackMailbox<'a, T>>::Output;
    fn send_ref_mut(&'a mut self, msg: &'a mut T) -> <Self as StackMailbox<'a, T>>::Output;
}

pub struct CoroutineMailbox<Co>(Co);
impl<'a, T, Co> Mailbox<'a, T> for CoroutineMailbox<Co>
where
    Co: Unpin + Coroutine<T>,
{
    type Output = CoroutineState<Co::Yield, Co::Return>;
    fn try_send(&'a mut self, msg: T) -> Self::Output {
        Pin::new(&mut self.0).resume(msg)
    }
}

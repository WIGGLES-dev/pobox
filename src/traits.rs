use std::{future::Future, marker::PhantomData};

pub enum SendError<T> {
    Full(T),
}
pub trait Sender<T>: Sized {
    fn try_send(self, value: T) -> Result<Self, SendError<T>>;
    fn capacity(&self) -> usize;
}
pub trait AsyncSender<T>: Sized {
    fn async_send(self, value: T) -> impl Future<Output = Result<Self, SendError<T>>>;
}
pub trait AsyncSendSender<T>: Sized {
    fn async_send(self, value: T) -> impl Send + Future<Output = Result<Self, SendError<T>>>;
}
pub trait AsyncSyncSender<T>: Sized {
    fn async_send(self, value: T) -> impl Sync + Future<Output = Result<Self, SendError<T>>>;
}
pub trait AsyncSenderSyncSender<T>: Sized {
    fn async_send(self, value: T)
    -> impl Send + Sync + Future<Output = Result<Self, SendError<T>>>;
}
pub trait BlockingSender<T>: Sized {
    fn blokcing_send(self, value: T) -> Result<Self, SendError<T>>;
}
pub trait RefSender<T> {
    fn try_send(&self) -> Result<(), SendError<T>>;
}
pub trait RefMutSender<T> {
    fn try_send(&mut self) -> Result<(), SendError<T>>;
}

pub struct RefSenderAdapter<T>(T);
pub struct RefmutSenderAdapter<T>(T);

pub enum ReceiveError<T> {
    Closed,
    Empty(T),
}
pub trait Receiver<T>: Sized {
    fn try_recv(self) -> Result<(Self, T), ReceiveError<Self>>;
}
pub trait AsyncReceiver<T>: Sized {
    fn async_recv(self) -> impl Future<Output = Result<(Self, T), ReceiveError<Self>>>;
}
pub trait AsyncSendReceiver<T>: Sized {
    fn async_recv(self) -> impl Send + Future<Output = Result<(Self, T), ReceiveError<Self>>>;
}
pub trait AsyncSyncReceiver<T>: Sized {
    fn async_rev(self) -> impl Sync + Future<Output = Result<(Self, T), ReceiveError<Self>>>;
}
pub trait AsyncSendSyncReceiver<T>: Sized {
    fn async_recv(
        self,
    ) -> impl Send + Sync + Future<Output = Result<(Self, T), ReceiveError<Self>>>;
}
pub trait BlockingReceiver<T>: Sized {
    fn blocking_recv(self) -> Result<(Self, T), ReceiveError<Self>>;
}
pub trait RefReceiver<T> {
    fn try_recv(&self) -> Result<(), ReceiveError<T>>;
}
pub trait RefMutReceiver<T> {
    fn try_recv(&mut self) -> Result<(), ReceiveError<T>>;
}

pub struct RefReceiverAdapter<T>(T);
pub struct RefMutReceiverAdapter<T>(T);

pub trait Input {
    type Receiver<T>: Receiver<T>;
}
pub trait Output {
    type Sender<T>: Sender<T>;
}

pub trait Channel: Input + Output {
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);
    fn bounded<T>(capacity: usize) -> (Self::Sender<T>, Self::Receiver<T>) {
        Self::channel()
    }
}

pub trait Transport: Channel {
    type Message;
}

pub trait Twin<T> {
    type Twin;
}

pub trait IsInput<T>: Receiver<T> + Twin<T> {}
pub trait IsOutput<T>: Sender<T> + Twin<T> {}

/// An expression that outputs are compatible allowing you to use the struct [Forwarded<T,U>] to generically represent
/// nested message passing and actors in message payloads
///
/// For example
///
/// message! {
///     Child<O: Output> {
///     
///     }
/// }
/// message! {
///     Parent<O: Output> {
///         SendToChild { child: Child<Forwarded<O>> } -> ()
///     }
/// }
///
/// where there has been a blanket implementation of Forwardable of ParentOutput to ChildOutput. It also supports the reverse,
/// of ChildOutput to ParentOutput.
pub trait Forwardable {}

pub struct Forwarded<T, U = ()> {
    phantom: PhantomData<(T, U)>,
}

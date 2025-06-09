use tokio::sync::{mpsc, oneshot};

pub use tokio::task;

use crate::{
    rpc::{Dispatch, IntoDispatch},
    runner::RunnerMessage,
};

pub enum SendError {
    Closed,
}

pub enum ReplyError {}

pub struct SingleReply<T>(pub(crate) oneshot::Sender<T>);

pub struct MultiReply<T>(pub(crate) mpsc::Sender<T>);

pub struct Mailbox<T>(pub(crate) mpsc::UnboundedReceiver<T>);

pub struct Notify(pub(crate) tokio::sync::Notify);

pub struct Maildrop<T>(pub(crate) mpsc::UnboundedSender<T>);

impl<T> Maildrop<RunnerMessage<T>>
where
    T: Dispatch,
{
    pub fn send<M>(&self, msg: M) -> Result<(), SendError>
    where
        M: IntoDispatch<Dispatch = T>,
    {
        self.0
            .send(RunnerMessage::Message {
                priority: 0,
                actor: 0,
                message: msg.into_dispatch(),
            })
            .map_err(|_| SendError::Closed)
    }
}

impl<T> Clone for Maildrop<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn mail<T>() -> (Maildrop<T>, Mailbox<T>) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    (Maildrop(sender), Mailbox(receiver))
}

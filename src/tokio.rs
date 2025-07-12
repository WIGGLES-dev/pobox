use std::process::Output;

use crate::{
    Actor, ActorRef, Service,
    mailbox::{AsyncMailbox, BlockingMailbox, Mailbox},
};

pub struct TokioMailbox<T>(tokio::sync::mpsc::Sender<T>);

impl<T> Clone for TokioMailbox<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> From<tokio::sync::mpsc::Sender<T>> for TokioMailbox<T> {
    fn from(value: tokio::sync::mpsc::Sender<T>) -> Self {
        Self(value)
    }
}

impl<T> TokioMailbox<T> {
    pub fn bounded(buffer: usize) -> (TokioMailbox<T>, tokio::sync::mpsc::Receiver<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(buffer);
        (TokioMailbox(sender), receiver)
    }
}

impl<'a, T> Mailbox<'a, T> for TokioMailbox<T> {
    type Output = Result<(), tokio::sync::mpsc::error::TrySendError<T>>;
    fn try_send(&'a mut self, msg: T) -> Self::Output {
        self.0.try_send(msg)
    }
}

impl<'a, T> BlockingMailbox<'a, T> for TokioMailbox<T> {
    type Output = Result<(), tokio::sync::mpsc::error::SendError<T>>;
    fn send_blocking(&'a mut self, msg: T) -> <Self as BlockingMailbox<'a, T>>::Output {
        self.0.blocking_send(msg)
    }
}

impl<'a, T> AsyncMailbox<'a, T> for TokioMailbox<T> {
    type Output = impl Future<Output = Result<(), tokio::sync::mpsc::error::SendError<T>>>;
    fn send(&'a mut self, msg: T) -> <Self as AsyncMailbox<'a, T>>::Output {
        self.0.send(msg)
    }
}

pub struct TokioSingleReply<T>(tokio::sync::oneshot::Sender<T>);

impl<T> From<tokio::sync::oneshot::Sender<T>> for TokioSingleReply<T> {
    fn from(value: tokio::sync::oneshot::Sender<T>) -> Self {
        Self(value)
    }
}

/// Spawn an actor in a tokio runtime, sequential queries will be executed at the same time using scoped threads
pub fn spawn_tokio<T, S>(buffer: usize, mut state: T) -> ActorRef<'static, T, S>
where
    T: 'static + Send + Sync + Actor<Mailbox<'static, S> = TokioMailbox<S>>,
    S: 'static + Send + Service<T>,
{
    let (mailbox, mut receiver) = TokioMailbox::bounded(buffer);
    tokio::spawn(async move {
        let mut messages: Vec<S> = vec![];

        loop {
            if receiver.recv_many(&mut messages, buffer).await == 0 {
                break;
            }

            let mut incoming = messages.drain(0..);

            let mut batch: Option<Vec<S>> = Some(vec![]);
            while let Some(msg) = incoming.next() {
                let is_mutation = msg.is_mutation();

                if is_mutation {
                    if let Some(mut batch) = batch.take() {
                        match tokio::task::spawn_blocking(move || {
                            std::thread::scope(|scope| {
                                for msg in batch.drain(0..) {
                                    scope.spawn(|| {
                                        S::handle_query(msg, &state, ());
                                    });
                                }
                            });
                            state
                        })
                        .await
                        {
                            Ok(pass) => state = pass,
                            Err(_) => {
                                return;
                            }
                        }
                    }
                }
                S::handle_mutation(msg, &mut state, ());
            }
        }
    });
    ActorRef::new(mailbox)
}

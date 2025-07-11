use std::marker::PhantomData;

use crate::traits::*;

pub struct TokioSender<T>(tokio::sync::mpsc::Sender<T>);
impl<T> Twin<T> for TokioSender<T> {
    type Twin = TokioReceiver<T>;
}

impl<T> Sender<T> for TokioSender<T> {
    fn capacity(&self) -> usize {
        10
    }
    fn try_send(self, value: T) -> Result<Self, SendError<T>> {
        use tokio::sync::mpsc::error::TrySendError::{Closed, Full};
        match self.0.try_send(value) {
            Ok(_) => Ok(self),
            Err(Closed(value) | Full(value)) => Err(SendError::Full(value)),
        }
    }
}
impl<T> AsyncSendSender<T> for TokioSender<T>
where
    T: Send,
{
    fn async_send(self, value: T) -> impl Send + Future<Output = Result<Self, SendError<T>>> {
        use tokio::sync::mpsc::error::SendError;

        async move {
            match self.0.send(value).await {
                Ok(_) => Ok(self),
                Err(SendError(value)) => Err(crate::traits::SendError::Full(value)),
            }
        }
    }
}

pub struct TokioReceiver<T>(tokio::sync::mpsc::Receiver<T>);
impl<T> Twin<T> for TokioReceiver<T> {
    type Twin = TokioSender<T>;
}

impl<T> Receiver<T> for TokioReceiver<T> {
    fn try_recv(mut self) -> Result<(Self, T), ReceiveError<Self>> {
        match self.0.try_recv() {
            Ok(msg) => Ok((self, msg)),
            Err(_) => Err(ReceiveError::Empty(self)),
        }
    }
}
impl<T> AsyncSendReceiver<T> for TokioReceiver<T>
where
    T: Send,
{
    fn async_recv(mut self) -> impl Send + Future<Output = Result<(Self, T), ReceiveError<Self>>> {
        async move {
            match self.0.recv().await {
                Some(msg) => Ok((self, msg)),
                None => Err(ReceiveError::Empty(self)),
            }
        }
    }
}

pub struct TokioTransport<M = ()> {
    phantom: PhantomData<M>,
}
impl<M> Input for TokioTransport<M> {
    type Receiver<T> = TokioReceiver<T>;
}

impl<M> Output for TokioTransport<M> {
    type Sender<T> = TokioSender<T>;
}

impl<M> Channel for TokioTransport<M> {
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(10);
        (TokioSender(sender), TokioReceiver(receiver))
    }
}
impl<M> Transport for TokioTransport<M> {
    type Message = M;
}

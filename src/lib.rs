#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(macro_metavar_expr_concat)]

mod dsl;
mod traits;

use std::marker::PhantomData;

use serde::Deserializer;

pub use traits::*;

pub fn serialize<S, T>(output: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    todo!()
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
{
    todo!()
}

pub trait Message {
    type Output;
}

pub struct GatewayInject<'a, 'de, M> {
    phantom: PhantomData<(&'a M, &'de M)>,
}
impl<'a, 'de, M> serde::de::DeserializeSeed<'a> for GatewayInject<'a, 'de, M>
where
    M: Message,
{
    type Value = M;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'a>,
    {
        todo!()
    }
}

pub struct Gateway<M: Message> {
    phantom: PhantomData<M>,
}

pub struct Actor<T>
where
    T: Transport,
{
    sender: Option<T::Sender<T::Message>>,
}

impl<T> Actor<T>
where
    T: Transport,
{
    pub fn spawn() -> (Self, T::Receiver<T::Message>) {
        let (sender, reciever) = T::channel();

        (
            Self {
                sender: Some(sender),
            },
            reciever,
        )
    }
    pub fn try_send(&mut self, msg: T::Message) {
        if let Some(sender) = self.sender.take() {
            match sender.try_send(msg) {
                Ok(sender) => {
                    self.sender = Some(sender);
                }
                Err(_) => {}
            }
        }
    }
}

impl<T> Actor<T>
where
    T: Transport,
    T::Sender<T::Message>: AsyncSendSender<T::Message>,
{
    pub async fn async_send(&mut self, msg: T::Message) {
        if let Some(sender) = self.sender.take() {
            match sender.async_send(msg).await {
                Ok(sender) => {
                    self.sender = Some(sender);
                }
                Err(_) => {}
            }
        }
    }
}

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "kanal")]
pub mod kanal;

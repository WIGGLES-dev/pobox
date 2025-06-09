use std::ops::{Deref, DerefMut};

use crate::{actor::ActorRef, rpc::Dispatch};

pub enum ProxyError {
    AlreadyProxied,
}

pub trait Proxyable<T>
where
    T: Dispatch,
{
    /// replaces a ProxyRef that has not already been proxied with the proxies internal behaviour
    fn proxy(&mut self, target: &mut ProxyRef<T>) -> Result<(), ProxyError>;
}

pub struct Proxy<T>
where
    T: Dispatch,
{
    inner: Vec<ActorRef<T>>,
    handler: ActorRef<T>,
}

impl<T> Proxyable<T> for Proxy<T>
where
    T: Dispatch,
{
    fn proxy(&mut self, target: &mut ProxyRef<T>) -> Result<(), ProxyError> {
        match target {
            ProxyRef::Proxy(_, _) => Err(ProxyError::AlreadyProxied),
            ProxyRef::Pure(actor) => {
                let id = self.inner.len();
                self.inner.push(actor.clone());
                *target = ProxyRef::Proxy(id, self.handler.clone());
                Ok(())
            }
        }
    }
}

/// An ActorRef that may be replaced with another Actor that implements the same interface.
/// ProxyRef::Proxy has space to store identity to support single mailbox proxies that know
/// how to forward messages back to the things they've proxied, when they've stored them internally.
/// see [crate::proxy::Proxy] for the a default implementation of holding that state in a single channel
pub enum ProxyRef<T>
where
    T: Dispatch,
{
    /// An ActorRef that may have been swapped by a proxy, but has not
    Pure(ActorRef<T>),
    /// An ActorRef whose implementation is no longer garunteed to be the original
    Proxy(usize, ActorRef<T>),
}

impl<T> Deref for ProxyRef<T>
where
    T: Dispatch,
{
    type Target = ActorRef<T>;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pure(v) => v,
            Self::Proxy(_, v) => v,
        }
    }
}

impl<T> DerefMut for ProxyRef<T>
where
    T: Dispatch,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pure(v) => v,
            Self::Proxy(_, v) => v,
        }
    }
}

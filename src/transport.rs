use std::{collections::HashSet, marker::PhantomData};

use crate::{actor::ActorRef, rpc::Dispatch};

pub enum InvokeError {}

pub struct Invoke {
    pub id: usize,
    pub method: u16,
    pub kind: u32,
    pub error: InvokeError,
}

pub struct Identity {
    node_id: usize,
}

pub struct TransportRuntime {
    myself: Identity,
    peers: HashSet<Identity>,
}

/// A special kind of actor that sits at node gateways, it is a singleton type
pub trait NodeGateway: Send + Sync + Sized + 'static {
    fn incoming(&mut self, msg: &[u8]);
}

pub struct SerdeGateway<S, D, T>
where
    T: Dispatch,
{
    deserializer: D,
    serializer: S,
    actor: ActorRef<T>,
}

impl<S, D, T> NodeGateway for SerdeGateway<S, D, T>
where
    S: serde::Serialize + Send + Sync + Sized + 'static,
    D: for<'de> serde::Deserializer<'de> + Send + Sync + Sized + 'static,
    T: Dispatch
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + crate::rpc::Dispatch
        + Send
        + Sync
        + Sized
        + 'static,
    T::State: Send + Sync + Sized + 'static,
{
    fn incoming(&mut self, msg: &[u8]) {}
}

pub struct AxumHttpTransport {}

pub struct AxumWsTransport {}

pub struct WasmBindgenTranpsort {}

pub struct ReqwestTransport {}

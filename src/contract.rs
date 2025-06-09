use crate::{actor::ActorRef, rpc::Dispatch};

/// a trait that is meant to model a contract between two actors. this is meant to solve a contextual use case
/// where one actor wants to give a priviledged permite to another actor if its meant some kind of authorization.
///
/// for example if Actor A receives a request from Actor B to acquire a permit to do some kind of special
/// thing, the actor doesn't have an easy way to remember that its allowed Actor B to do so without storing
/// it inline.
///
/// this is an attempt to standardize the ability to store contracts inline in an actor in a composable way.
///
/// actors that want to allow contracts simply need to embed these types in there interface to get this functionality
pub trait Contract {
    type Dispatch: Dispatch;
}

// a permit to send a message to an actor, that holds onto some state that doesn't need to be
// passed around in channel plumbing.
pub struct ActorPermit<T>
where
    T: Contract,
{
    state: T,
    sink: ActorRef<T::Dispatch>,
}

/// holds all the contracts this actor has granted for this particular contract
pub struct Contracts<T>
where
    T: Contract,
{
    permits: Vec<ActorPermit<T>>,
}

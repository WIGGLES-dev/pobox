pub mod isolated;
pub mod lock;
mod sharded;

use crate::{
    actor::ActorRef,
    concurrency::{Maildrop, Notify, SingleReply},
    rpc::Dispatch,
};

pub trait Runner {}

pub struct TaskRunnerHandle<T>
where
    T: Dispatch,
{
    pub sender: Maildrop<RunnerMessage<T>>,
}

pub struct ThreadRunnerHandle<T>
where
    T: Dispatch,
{
    pub handle: std::thread::JoinHandle<()>,
    pub sender: Maildrop<RunnerMessage<T>>,
}

/// a way for runners to support maybe synchronous code
pub enum Execution<S, A> {
    Sync(S),
    Async(A),
}

pub trait RunSync {}

pub trait RunAsync {}

/// signals to affect how the default actor runtime handles messages
pub enum RunnerMessage<T>
where
    T: Dispatch,
{
    /// a request to pause this actor, optionally returning the state to the caller
    Kill {
        actor: usize,
        reply: Option<SingleReply<T>>,
    },
    /// a request to pause this actor returning the state to the caller
    Pause {
        actor: usize,
        reply: Option<SingleReply<T>>,
    },
    /// a request to resume a paused actor, with the passed state
    Resume {
        actor: usize,
        state: T::State,
    },
    /// a standard message with a user defined priority
    Message {
        priority: usize,
        actor: usize,
        message: T,
    },
    Lock {
        actor: usize,
        notify: Notify,
    },
    Unlock {
        actor: usize,
    },
    /// Spawn a new actor into this runner, awaiting the [ActorRef] when the runner completes this message
    Spawn {
        /// control the affinity of how this actor will be moved about, the runner will try to move actors of similar
        /// affinity to the same shard
        affinity: usize,
        /// the state of your actor, don't worry you can get it back, eventually
        state: T::State,
        reply: SingleReply<ActorRef<T>>,
    },
}

/// different strategies for dropping messages
pub enum MessageDropping {
    /// never drop a message
    Forbidden,
    /// always drop messages if the actor can't keep up
    Always,
    /// strike a balance between performance and message dropping
    Optimized,
}

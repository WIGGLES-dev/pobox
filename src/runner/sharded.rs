use std::collections::HashMap;

use crate::{
    actor::ActorCell,
    concurrency::{mail, task},
    rpc::Dispatch,
};

use super::{MessageDropping, RunnerMessage, TaskRunnerHandle};

enum ActorState<T>
where
    T: Dispatch,
{
    Local(Box<ActorCell<T::State>>),
    LocalLocked(Box<ActorCell<T::State>>, Vec<RunnerMessage<T>>),
    Shard(usize),
    ShardLocked(usize, Vec<RunnerMessage<T>>),
}

impl<T> ActorState<T>
where
    T: Dispatch,
{
    fn lock_local(&mut self) {
        // Use std::mem::replace to swap out self temporarily
        let new_state = match std::mem::replace(self, Self::Shard(0)) {
            Self::Local(state) => {
                // Create the LocalLocked variant with the existing state and an empty Vec
                Self::LocalLocked(state, Vec::new())
            }
            other => {
                // Put the original back if it was not Local
                *self = other;
                return;
            }
        };
        *self = new_state;
    }
}

/// opts for configuring the behaviour of a sharded runner
pub struct ShardedRunnerOpts<T>
where
    T: Dispatch,
{
    /// how many messages should the runner process at a time?
    chunk_size: usize,
    /// what is the absolute maximum number of shards this runner is allowed to produce
    max_shards: usize,
    /// other top level [ShardedRunner] instances that you want delegated to before this runner produces shards
    peers: Vec<TaskRunnerHandle<T>>,
    /// this runners tolerance level for dropping messages, each concrete [Dispatch::State] type gets its own runner
    /// so you can configure this per domain object
    message_dropping: MessageDropping,
}
/// A runner that is capable of spinning off shards when load gets too high
pub struct ShardedRunner<T>
where
    T: Dispatch,
{
    /// at the top level a [ShardedRunner] must have a reference to every actor type its seen, as it serves as a router
    /// the underlying Actor is either going to be on of [ActorState] (local to this shard, locked on this shard, on a shard, or locked on some shard)
    state: HashMap<usize, ActorState<T>>,
    /// only one [ShardedRunner] is the root, the root runner spawns many child runners to forward messages to
    root: bool,
    /// the child shards, while the root shard still has to process every mailbox message, it can stay on the hot path simply by dumping
    /// messages into the shards its spawned. If load gets too high offload those shards to a remote node
    shards: Vec<TaskRunnerHandle<T>>,
}

impl<T> ShardedRunner<T>
where
    T: Dispatch + Send + 'static,
    T::State: Send,
{
    pub fn run(opts: ShardedRunnerOpts<T>) -> TaskRunnerHandle<T> {
        let (sender, mut receiver) = mail::<RunnerMessage<T>>();
        async move {
            let mut runner = Self {
                state: HashMap::new(),
                root: true,
                shards: vec![],
            };

            let mut messages = vec![];

            loop {
                let tick_messages_received =
                    receiver.0.recv_many(&mut messages, opts.chunk_size).await;

                for msg in messages.drain(0..opts.chunk_size) {
                    match msg {
                        RunnerMessage::Kill { actor, reply } => {}
                        RunnerMessage::Pause { actor, reply } => {}
                        RunnerMessage::Resume { actor, state } => {}
                        RunnerMessage::Message {
                            priority,
                            actor,
                            message,
                        } => {
                            let actor_state = runner.state.get_mut(&actor);
                            match actor_state {
                                Some(ActorState::Local(state)) => {
                                    match message.run_mut(state.value.get_mut()) {
                                        Ok(_) => {}
                                        Err(_) => {}
                                    }
                                }
                                Some(ActorState::Shard(shard)) => {
                                    match runner.shards[*shard].sender.0.send(
                                        RunnerMessage::Message {
                                            priority,
                                            actor,
                                            message,
                                        },
                                    ) {
                                        Ok(_) => {}
                                        Err(_) => {}
                                    }
                                }
                                Some(ActorState::LocalLocked(_, overflow)) => {
                                    overflow.push(RunnerMessage::Message {
                                        priority,
                                        actor,
                                        message,
                                    });
                                }
                                Some(ActorState::ShardLocked(_, overflow)) => {
                                    overflow.push(RunnerMessage::Message {
                                        priority,
                                        actor,
                                        message,
                                    });
                                }
                                None => {}
                            }
                        }
                        RunnerMessage::Lock { actor, notify } => {
                            let mut actor_state = runner.state.get_mut(&actor);
                            match actor_state {
                                Some(
                                    ActorState::LocalLocked(_, _) | ActorState::ShardLocked(_, _),
                                ) => {
                                    // error can't lock actor twice
                                }
                                Some(ActorState::Local(state)) => {
                                    actor_state.unwrap().lock_local();
                                }
                                Some(ActorState::Shard(shard)) => {}
                                _ => {}
                            }
                        }
                        RunnerMessage::Unlock { actor } => {}
                        RunnerMessage::Spawn {
                            affinity,
                            state,
                            reply,
                        } => {
                            let boxed_state = Box::new(ActorCell::new(state));
                            let ptr: *const ActorCell<<T as Dispatch>::State> =
                                Box::as_ref(&boxed_state)
                                    as *const ActorCell<<T as Dispatch>::State>;
                            let id = ptr as usize;
                            runner.state.insert(id, ActorState::Local(boxed_state));
                        }
                    }
                }

                if messages.len() != 0 {
                    // we'res getting messages faster than we can process them
                }
            }
        };
        TaskRunnerHandle { sender }
    }
}

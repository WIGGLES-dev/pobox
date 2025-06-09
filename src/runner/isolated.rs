use std::marker::PhantomData;

use crate::{concurrency::mail, rpc::Dispatch};

use super::{RunnerMessage, TaskRunnerHandle, ThreadRunnerHandle};

pub struct IsolatedRunnerOpts<T>
where
    T: Dispatch,
{
    pub chunk_size: usize,
    pub state: T::State,
}

pub struct IsolatedRunner<T>
where
    T: Dispatch,
{
    phantom: PhantomData<T>,
}

pub enum ActorState {
    Locked,
    Unlocked,
}

impl<T> IsolatedRunner<T>
where
    T: Dispatch + Send + 'static,
    T::State: Send + 'static,
{
    pub fn spawn_sync(opts: IsolatedRunnerOpts<T>) -> ThreadRunnerHandle<T> {
        if T::ASYNC {
            panic!("cannot spawn async actor in sync runner");
        }

        let (sender, mut receiver) = mail::<RunnerMessage<T>>();
        let handle = std::thread::spawn(move || {
            let mut state = opts.state;
            let mut messages = vec![];
            let tick_messages_processed = receiver
                .0
                .blocking_recv_many(&mut messages, opts.chunk_size);

            for message in messages.drain(0..opts.chunk_size) {
                match message {
                    RunnerMessage::Kill { actor, reply } => {
                        return ();
                    }
                    RunnerMessage::Pause { actor, reply } => {}
                    RunnerMessage::Resume { actor, state } => {}
                    RunnerMessage::Message {
                        priority,
                        actor,
                        message,
                    } => {
                        match T::run_mut(message, &mut state) {
                            Ok(_) => {}
                            Err(_) => {}
                        };
                    }
                    RunnerMessage::Lock { actor, notify } => {}
                    RunnerMessage::Unlock { actor } => {}
                    RunnerMessage::Spawn {
                        affinity,
                        state,
                        reply,
                    } => {}
                }
            }
            unreachable!()
        });
        ThreadRunnerHandle { handle, sender }
    }

    pub fn spawn(opts: IsolatedRunnerOpts<T>) -> TaskRunnerHandle<T> {
        let (sender, mut receiver) = mail::<RunnerMessage<T>>();

        async move {
            let mut state = opts.state;
            let mut messages = vec![];

            loop {
                let tick_messages_processed =
                    receiver.0.recv_many(&mut messages, opts.chunk_size).await;

                for message in messages.drain(0..opts.chunk_size) {
                    match message {
                        RunnerMessage::Kill { actor, reply } => {
                            return ();
                        }
                        RunnerMessage::Pause { actor, reply } => todo!(),
                        RunnerMessage::Resume { actor, state } => todo!(),
                        RunnerMessage::Message {
                            priority,
                            actor,
                            message,
                        } => match T::spawn_mut(message, &mut state).await {
                            Ok(_) => {}
                            Err(_) => {}
                        },
                        RunnerMessage::Lock { actor, notify } => todo!(),
                        RunnerMessage::Unlock { actor } => todo!(),
                        RunnerMessage::Spawn {
                            affinity,
                            state,
                            reply,
                        } => todo!(),
                    }
                }
            }
        };

        TaskRunnerHandle { sender }
    }
}

use std::marker::PhantomData;

use axum::{extract::State, routing::MethodRouter};

use crate::{Actor, ActorRef};

#[cfg(feature = "axum-ws")]
pub struct WebsocketMount<S> {
    phantom: PhantomData<S>,
}

#[cfg(feature = "axum-ws")]
impl<S> WebsocketMount<S> {
    pub fn mount<F, Fut, T, Ser>(f: F) -> MethodRouter<S>
    where
        T: Actor,
        F: 'static + Send + Sync + Clone + FnOnce(S) -> Fut,
        Fut: Future<Output = ActorRef<'static, T, Ser>>,
        S: 'static + Send + Sync + Clone,
    {
        use axum::{extract::WebSocketUpgrade, routing::get};
        get(|state: State<S>, upgrade: WebSocketUpgrade| async move {
            let actor = f(state.0);
        })
    }
}

pub struct RouteMount<S> {
    phantom: PhantomData<S>,
}

impl<S> RouteMount<S> {
    fn mount(self) {}
}

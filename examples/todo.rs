#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(macro_metavar_expr_concat)]

use std::ops::Coroutine;

use pobox::*;
use serde::{Deserialize, Serialize};

fn main() {}

message! {
    #[derive(Deserialize, Serialize)]
    pub Todos<O: Output> {
        TodosSince { since: String } -> String,
        OpenTodos -> String
    }
}

pub trait TodosActorArgs<O> {}
fn todos_actor<O>(args: impl TodosActorArgs<O>) -> impl Coroutine<Todos<O>, Yield = ()>
where
    O: TodosOutput,
{
    #[coroutine]
    |msg| {}
}

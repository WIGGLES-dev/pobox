#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(macro_metavar_expr_concat)]
#![feature(impl_trait_in_assoc_type)]

use pobox::{tokio::TokioMailbox, *};

fn main() {
    let (mailbox, receiver) = TokioMailbox::bounded(100);
    let actor_state = Todos {};

    let mut actor_ref = ActorRef::<Todos, TodosService>::new(mailbox);
}

struct Todos {}
impl Actor for Todos {
    type Mailbox<'a, S> = TokioMailbox<S>;
}

struct AddTodo {}
impl HasReply<AddTodo> for Todos {
    type Reply = ();
}
impl<S> Mutation<AddTodo, S> for Todos {
    fn handle(&mut self, msg: AddTodo) -> Self::Reply {}
}

pub struct RemoveTodo {}
impl HasReply<RemoveTodo> for Todos {
    type Reply = ();
}

impl<S> Mutation<RemoveTodo, S> for Todos
where
    // we use this to make sure the message GetOpenTodos is sendable to Todos
    GetOpenTodos: ServiceMember<S>,
{
    fn handle(&mut self, msg: RemoveTodo) -> Self::Reply {}
}

pub struct GetOpenTodos {}
impl HasReply<GetOpenTodos> for Todos {
    type Reply = ();
}

impl<S> Query<GetOpenTodos, S> for Todos {
    fn handle(&self, q: GetOpenTodos) -> Self::Reply {}
}

service! {
    TodosService {
        Queries {
            GetOpenTodos,
        }
        Mutations {
            AddTodo,
            RemoveTodo,
        }
    }
}

use pobox::*;

fn main() {}

#[derive(Clone)]
pub struct Todo {}

#[derive(BorrowMask)]
pub struct Todos {
    locked: bool,
    todos: Vec<Todo>,
}

#[dispatch]
impl Todos {
    pub fn get_todos(Self { ref locked, .. }: &mut Self) -> Vec<Todo> {
        *locked = true;
        vec![]
    }
    pub fn insert_todo(Self { todos, .. }: &mut Self, todo: Todo, todo2: &Todo) {
        todos.push(todo);
    }

    pub async fn get_remote_todos(Self { todos, .. }: &Self) {}
}

#[test]
fn test() {
    use mailbox::{
        rpc::{Dispatch, IntoDispatch},
        runner::isolated::{IsolatedRunner, IsolatedRunnerOpts},
    };
    use std::borrow::Cow;

    let mut todos = Todos {
        locked: false,
        todos: vec![],
    };

    todos.get_todos();

    let msg = todos_mailbox::InsertTodo {
        todo: Todo {},
        todo2: Cow::Owned(Todo {}),
    };

    let runner = IsolatedRunner::spawn(IsolatedRunnerOpts {
        chunk_size: 25,
        state: todos,
    });

    runner.sender.send(msg);
}

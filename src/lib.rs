#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]
#![feature(macro_metavar_expr_concat)]
#![feature(impl_trait_in_assoc_type)]

use std::marker::PhantomData;

use crate::mailbox::{AsyncMailbox, BlockingMailbox, Mailbox, OwnedMailbox};

#[cfg(feature = "axum")]
pub mod axum;
#[cfg(feature = "tokio")]
pub mod tokio;
#[cfg(feature = "tower")]
pub mod tower;

pub mod mailbox;
pub mod service;

pub struct ActorContext<'a, T, S>
where
    T: Actor,
{
    mailbox: T::Mailbox<'a, S>,
}
pub trait Actor {
    type Mailbox<'a, S>: mailbox::Mailbox<'a, S>;
}

pub trait Reply {}

impl Reply for () {}

pub trait HasReply<T> {
    type Reply: Reply;
}

pub struct QueryContext<'a, T, S>
where
    T: Actor,
{
    state: &'a T,
    actor: ActorContext<'a, T, S>,
}
pub trait Query<T, S>: HasReply<T> {
    fn handle(&self, query: T) -> Self::Reply;
}

pub struct MutationContext<'a, T, S>
where
    T: Actor,
{
    state: &'a mut T,
    actor: ActorContext<'a, T, S>,
}
pub trait Mutation<T, S>: HasReply<T> {
    fn handle(&mut self, msg: T) -> Self::Reply;
}

pub trait ReplyForQuery<T> {}
impl<T> ReplyForQuery<T> for () {}
pub trait ReplyForMutation<T> {}
impl<T> ReplyForMutation<T> for () {}

pub trait ServiceMember<S>: Into<S> + std::convert::TryFrom<S> {}
pub trait Service<T> {
    fn handle_query<R>(self, actor: &T, reply: R)
    where
        R: ReplyForQuery<T>;
    fn handle_mutation<R>(self, actor: &mut T, reply: R)
    where
        R: ReplyForMutation<T>;
    fn is_mutation(&self) -> bool;
}

pub struct ServiceQuery<Q> {
    query: Q,
}

pub struct ServiceMutation<M> {
    mutation: M,
}

#[macro_export]
macro_rules! actor {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident<$generic:ident>
    ) => {
        $(#[$attr])*
        $vis struct $name<$generic>($generic);
        impl<$generic> ::std::ops::Deref for $name<$generic> {
            type Target = $generic;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<$generic> ::std::ops::DerefMut for $name<$generic> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

#[macro_export]
macro_rules! service {
    (
        $(#[$container_attr:meta])*
        $vis:vis $name:ident {
            Queries {
                $(
                    $(#[$query_variant_attr:meta])*
                    $query:ident
                ),* $(,)?
            }

            Mutations {
                $(
                    $(#[$mutation_variant_attr:meta])*
                    $mutation:ident
                ),* $(,)?
            }

        }
    ) => {
        $( #[ $container_attr ] )*
        $vis enum $name {
            $(
                $( #[ $query_variant_attr ] )*
                $query($query)
            ,)*

            $(
                $( #[ $mutation_variant_attr ] )*
                $mutation($mutation)
            ,)*
        }

        impl<T> ::pobox::Service<T> for $name
        where
        $( T: ::pobox::Query<$query, $name>, )*
        $( T: ::pobox::Mutation<$mutation, $name>, )*
        {
            fn handle_query<R>(self, actor: &T, rpely: R)
            where R: ::pobox::ReplyForQuery<T>
            {
                match self {
                    $(
                        Self::$query(query) => {

                        },
                    )*
                    _ => {}
                }
            }
            fn handle_mutation<R>(self, actor: &mut T, reply: R)
            where R: ::pobox::ReplyForMutation<T>
            {
                match self {
                    $(
                        Self::$mutation(mutation) => {},
                    )*
                    _ => {}
                }
            }
            fn is_mutation(&self) -> bool {
                match self {
                    $(
                        Self::$mutation(mutation) => true,
                    )*
                    _ => false
                }
            }
        }

        $(
            impl ::std::convert::TryFrom<$name> for $query {
                type Error = ();
                fn try_from(msg: $name) -> Result<Self, Self::Error> {
                    match msg {
                        $name::$query(query) => Ok(query),
                        _ => Err(())
                    }
                }
            }
            impl Into<$name> for $query {
                fn into(self) -> $name {
                    $name::$query(self)
                }
            }
            impl ::pobox::ServiceMember<$name> for $query {}
        )*

        $(
            impl ::std::convert::TryFrom<$name> for $mutation {
                type Error = ();
                fn try_from(msg: $name) -> Result<Self, Self::Error> {
                    match msg {
                        $name::$mutation(mutation) => Ok(mutation),
                        _ => Err(())
                    }
                }
            }
            impl Into<$name> for $mutation {
                fn into(self) -> $name {
                    $name::$mutation(self)
                }
            }
            impl ::pobox::ServiceMember<$name> for $mutation {}
        )*

        $vis trait ${concat($name, ServiceExt)} {}
        impl ${concat($name, ServiceExt)} for $name {}
    };
}

pub struct ActorRef<'a, T, S>
where
    T: Actor,
{
    mailbox: T::Mailbox<'a, S>,
    phantom: PhantomData<T>,
}

impl<'a, T, S> Clone for ActorRef<'a, T, S>
where
    T: Actor,
    <T as Actor>::Mailbox<'a, S>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mailbox: self.mailbox.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, S> ActorRef<'a, T, S>
where
    T: Actor,
    S: Service<T>,
{
    pub fn new(mailbox: T::Mailbox<'a, S>) -> Self {
        Self {
            mailbox,
            phantom: PhantomData,
        }
    }

    pub fn try_send<M>(
        &'a mut self,
        msg: M,
    ) -> <<T as Actor>::Mailbox<'a, S> as mailbox::Mailbox<'a, S>>::Output
    where
        M: ServiceMember<S> + HasReply<T>,
    {
        mailbox::Mailbox::try_send(&mut self.mailbox, msg.into())
    }

    pub fn send_blocking<M>(
        &'a mut self,
        msg: M,
    ) -> <<T as Actor>::Mailbox<'a, S> as mailbox::BlockingMailbox<'a, S>>::Output
    where
        M: ServiceMember<S> + HasReply<T>,
        <T as Actor>::Mailbox<'a, S>: mailbox::BlockingMailbox<'a, S>,
    {
        mailbox::BlockingMailbox::send_blocking(&mut self.mailbox, msg.into())
    }

    pub fn send_async<M>(&'a mut self, msg: M) -> impl Future<Output = ()>
    where
        M: ServiceMember<S> + HasReply<T>,
        <T as Actor>::Mailbox<'a, S>: mailbox::AsyncMailbox<'a, S>,
        <<T as Actor>::Mailbox<'a, S> as AsyncMailbox<'a, S>>::Output: Future<Output = ()>,
    {
        mailbox::AsyncMailbox::send(&mut self.mailbox, msg.into())
    }

    pub fn send_async_send<M>(&'a mut self, msg: M) -> impl Send + Future<Output = ()>
    where
        M: ServiceMember<S> + HasReply<T>,
        <T as Actor>::Mailbox<'a, S>: mailbox::AsyncMailbox<'a, S>,
        <<T as Actor>::Mailbox<'a, S> as AsyncMailbox<'a, S>>::Output: Send + Future<Output = ()>,
    {
        mailbox::AsyncMailbox::send(&mut self.mailbox, msg.into())
    }

    pub fn send_async_sync<M>(&'a mut self, msg: M) -> impl Sync + Future<Output = ()>
    where
        M: ServiceMember<S> + HasReply<T>,
        <T as Actor>::Mailbox<'a, S>: mailbox::AsyncMailbox<'a, S>,
        <<T as Actor>::Mailbox<'a, S> as AsyncMailbox<'a, S>>::Output: Sync + Future<Output = ()>,
    {
        mailbox::AsyncMailbox::send(&mut self.mailbox, msg.into())
    }

    pub fn send_async_send_sync<M>(&'a mut self, msg: M) -> impl Send + Sync + Future<Output = ()>
    where
        M: ServiceMember<S> + HasReply<T>,
        <T as Actor>::Mailbox<'a, S>: mailbox::AsyncMailbox<'a, S>,
        <<T as Actor>::Mailbox<'a, S> as AsyncMailbox<'a, S>>::Output:
            Send + Sync + Future<Output = ()>,
    {
        mailbox::AsyncMailbox::send(&mut self.mailbox, msg.into())
    }
}

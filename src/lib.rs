#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod actor;
pub mod borrow;
pub mod concurrency;
mod contract;
pub mod disjoint;
mod message;
pub mod proxy;
mod registry;
pub mod rpc;
pub mod runner;
#[cfg(test)]
mod tests;
mod transport;

pub use mailbox_macros::*;

pub mod prelude {
    pub use crate::{actor::ActorRef, borrow::BorrowMask};

    pub use mailbox_macros::*;
}

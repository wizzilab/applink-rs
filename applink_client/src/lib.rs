#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic_in_result_fn)]
#![deny(clippy::panic)]
#![deny(clippy::indexing_slicing)]

pub use applink_codec as codec;

pub mod amqp;
pub mod common;
pub mod http;
pub mod mqtt;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#![allow(unused)]

mod get_if_addrs;
mod dns;

#[macro_use]
extern crate log;

pub use get_if_addrs::*;
pub use dns::*;

mod events;
mod ids;
mod models;
mod money;
mod services;
mod result;

use money::Money;

pub use result::Result;

use std::fmt;

pub fn print_hello(name: impl fmt::Display) {
    println!("Hello, {name}!");
}


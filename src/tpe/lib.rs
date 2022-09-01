mod events;
mod ids;
mod models;
mod money;

use std::fmt;

pub fn print_hello(name: impl fmt::Display) {
    println!("Hello, {name}!");
}


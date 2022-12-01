use clap::{Parser, Subcommand};

mod actor;
mod target;

pub use actor::*;
pub use target::*;

#[derive(Parser, Debug)]
pub struct Arguments {
    #[arg(long, default_value_t = String::from("localhost"))]
    pub host: String,
    #[arg(long, default_value_t = 6174)]
    pub port: u32,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(name = "actors")]
    Actor(Actor),
    #[clap(name = "targets")]
    Target(Target),
}

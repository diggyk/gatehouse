use clap::{Args, Parser, Subcommand};

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
    #[clap(name = "targets")]
    Target(Target),
}

#[derive(Parser, Debug)]
pub struct Target {
    #[clap(subcommand)]
    pub target_cmds: TargetCmds,
}

#[derive(Subcommand, Debug)]
pub enum TargetCmds {
    Add(TargetCmdAddArgs),
    AddAction(TargetCmdAddActionArgs),
    RemoveAction(TargetCmdAddActionArgs),
    Remove(TargetCmdRemoveArgs),
}

#[derive(Args, Debug)]
pub struct TargetCmdAddArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long,
        short,
        required = false,
        help = "Repeat arg to specify multiple actions"
    )]
    pub actions: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TargetCmdRemoveArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
}

#[derive(Args, Debug)]
pub struct TargetCmdAddActionArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long,
        short,
        required = true,
        help = "Repeat arg to specify multiple actions"
    )]
    pub actions: Vec<String>,
}

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Actor {
    #[clap(subcommand)]
    pub actor_cmds: ActorCmds,
}

#[derive(Subcommand, Debug)]
pub enum ActorCmds {
    Add(ActorCmdAddArgs),
    Modify(ActorCmdModifyArgs),
    Remove(ActorCmdRemoveArgs),
    Search(ActorCmdSearchArgs),
}

#[derive(Args, Debug)]
pub struct ActorCmdAddArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long,
        short = 't',
        required = false,
        help = "Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub attribs: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ActorCmdModifyArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long = "at",
        required = false,
        help = "Attributes to add. Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub add_attribs: Vec<String>,
    #[arg(
        long = "rt",
        required = false,
        help = "Attributes to remove. Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub remove_attribs: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ActorCmdSearchArgs {
    #[arg(help = "Type (case-insensitive)", required = false)]
    pub typestr: Option<String>,
    #[arg(help = "Name (case-insensitive)", required = false)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct ActorCmdRemoveArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
}

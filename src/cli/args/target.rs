use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Target {
    #[clap(subcommand)]
    pub target_cmds: TargetCmds,
}

#[derive(Subcommand, Debug)]
pub enum TargetCmds {
    Add(TargetCmdAddArgs),
    Modify(TargetCmdModifyArgs),
    Remove(TargetCmdRemoveArgs),
    Search(TargetCmdSearchArgs),
}

#[derive(Args, Debug)]
pub struct TargetCmdAddArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long,
        short = 'a',
        required = false,
        help = "Repeat arg to specify multiple actions"
    )]
    pub actions: Vec<String>,
    #[arg(
        long,
        short = 't',
        required = false,
        help = "Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub attribs: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TargetCmdModifyArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
    #[arg(
        long = "aa",
        required = false,
        help = "Actions to add. Repeat arg to specify multiple actions"
    )]
    pub add_actions: Vec<String>,
    #[arg(
        long = "at",
        required = false,
        help = "Attributes to add. Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub add_attribs: Vec<String>,
    #[arg(
        long = "ra",
        required = false,
        help = "Actions to remove. Repeat arg to specify multiple actions"
    )]
    pub remove_actions: Vec<String>,
    #[arg(
        long = "rt",
        required = false,
        help = "Attributes to remove. Attribute of format '{key}:{val1},{val2},{val3}'"
    )]
    pub remove_attribs: Vec<String>,
}

#[derive(Args, Debug)]
pub struct TargetCmdSearchArgs {
    #[arg(help = "Type (case-insensitive)", required = false)]
    pub typestr: Option<String>,
    #[arg(help = "Name (case-insensitive)", required = false)]
    pub name: Option<String>,
}

#[derive(Args, Debug)]
pub struct TargetCmdRemoveArgs {
    #[arg(help = "Type (case-insensitive)")]
    pub typestr: String,
    #[arg(help = "Name (case-insensitive)")]
    pub name: String,
}

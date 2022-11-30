extern crate clap;

use clap::Parser;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

mod args;
mod cmds;

use crate::args::{Arguments, Commands, TargetCmds};
use crate::cmds::{add_target, modify_target, remove_target};

#[tokio::main]
async fn main() {
    let args = Arguments::parse();

    let mut client = GatehouseClient::connect(format!("http://{}:{}", args.host, args.port))
        .await
        .expect("Could not create client");

    match args.command {
        Commands::Target(target_args) => match target_args.target_cmds {
            TargetCmds::Add(args) => add_target(&mut client, args).await,
            TargetCmds::Modify(args) => modify_target(&mut client, args).await,
            TargetCmds::Remove(args) => remove_target(&mut client, args).await,
        },
    }
}

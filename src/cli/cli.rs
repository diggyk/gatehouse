extern crate clap;

use clap::Parser;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

mod args;
mod cmds;

use crate::args::{Arguments, Commands, TargetCmds};
use crate::cmds::{add_target, add_target_action, remove_target, remove_target_action};

#[tokio::main]
async fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);

    let mut client = GatehouseClient::connect(format!("http://{}:{}", args.host, args.port))
        .await
        .expect("Could not create client");

    match args.command {
        Commands::Target(target_args) => match target_args.target_cmds {
            TargetCmds::Add(args) => add_target(&mut client, args).await,
            TargetCmds::AddAction(args) => add_target_action(&mut client, args).await,
            TargetCmds::RemoveAction(args) => remove_target_action(&mut client, args).await,
            TargetCmds::Remove(args) => remove_target(&mut client, args).await,
        },
    }
}

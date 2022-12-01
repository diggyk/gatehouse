extern crate clap;

use clap::Parser;

use cmds::{add_actor, get_actors, get_targets, modify_actor, remove_actor};
use gatehouse::proto::base::gatehouse_client::GatehouseClient;

mod args;
mod cmds;

use crate::args::{ActorCmds, Arguments, Commands, TargetCmds};
use crate::cmds::{add_target, modify_target, remove_target};

#[tokio::main]
async fn main() {
    let args = Arguments::parse();

    let mut client = GatehouseClient::connect(format!("http://{}:{}", args.host, args.port))
        .await
        .expect("Could not create client");

    match args.command {
        Commands::Target(args) => match args.target_cmds {
            TargetCmds::Add(args) => add_target(&mut client, args).await,
            TargetCmds::Modify(args) => modify_target(&mut client, args).await,
            TargetCmds::Remove(args) => remove_target(&mut client, args).await,
            TargetCmds::Search(args) => get_targets(&mut client, args).await,
        },
        Commands::Actor(args) => match args.actor_cmds {
            ActorCmds::Add(args) => add_actor(&mut client, args).await,
            ActorCmds::Modify(args) => modify_actor(&mut client, args).await,
            ActorCmds::Remove(args) => remove_actor(&mut client, args).await,
            ActorCmds::Search(args) => get_actors(&mut client, args).await,
        },
    }
}

use tonic::transport::Channel;

use gatehouse::helpers;
use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use crate::args::{ActorCmdAddArgs, ActorCmdModifyArgs, ActorCmdRemoveArgs, ActorCmdSearchArgs};

use super::form_attributes;

pub async fn add_actor(client: &mut GatehouseClient<Channel>, args: ActorCmdAddArgs) {
    let attributes = form_attributes(&args.attribs);

    match helpers::add_actor(client, &args.name, &args.typestr, attributes).await {
        Ok(actor) => println!("Added {actor}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

pub async fn modify_actor(client: &mut GatehouseClient<Channel>, args: ActorCmdModifyArgs) {
    let add_attributes = form_attributes(&args.add_attribs);
    let remove_attributes = form_attributes(&args.remove_attribs);

    match helpers::modify_actor(
        client,
        &args.name,
        &args.typestr,
        add_attributes,
        remove_attributes,
    )
    .await
    {
        Ok(actor) => println!("Updated {actor}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

pub async fn get_actors(client: &mut GatehouseClient<Channel>, args: ActorCmdSearchArgs) {
    match helpers::get_actors(client, args.name, args.typestr).await {
        Ok(actors) => {
            println!("Got {} actors:", actors.len());
            for actor in actors {
                println!("{actor}");
            }
        }
        Err(err) => println!("Error: {err}"),
    }
}

pub async fn remove_actor(client: &mut GatehouseClient<Channel>, args: ActorCmdRemoveArgs) {
    match helpers::remove_actor(client, &args.name, &args.typestr).await {
        Ok(actor) => println!("Removed {actor}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

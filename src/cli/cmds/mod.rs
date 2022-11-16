use gatehouse::proto::targets::{
    AddTargetActionRequest, AddTargetRequest, RemoveTargetActionRequest, RemoveTargetRequest,
};
use tonic::transport::Channel;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use crate::args::{TargetCmdAddActionArgs, TargetCmdAddArgs, TargetCmdRemoveArgs};

pub async fn add_target(client: &mut GatehouseClient<Channel>, args: TargetCmdAddArgs) {
    let req = AddTargetRequest {
        name: args.name,
        typestr: args.typestr,
        actions: args.actions,
    };

    match client.add_target(req).await {
        Ok(response) => {
            let tgt = response.into_inner().target;
            if tgt.is_some() {
                println!("Added: {}", tgt.unwrap());
            } else {
                println!("Error: No target returned after adding");
            }
        }
        Err(err) => println!("Error: {}", err.message()),
    }
}

pub async fn add_target_action(
    client: &mut GatehouseClient<Channel>,
    args: TargetCmdAddActionArgs,
) {
    println!("{:?}", args);
    let req = AddTargetActionRequest {
        name: args.name,
        typestr: args.typestr,
        actions: args.actions,
    };

    match client.add_target_action(req).await {
        Ok(response) => {
            let tgt = response.into_inner().target;
            if tgt.is_some() {
                println!("Updated: {}", tgt.unwrap());
            } else {
                println!("Error: No target returned after adding");
            }
        }
        Err(err) => println!("Error: {}", err.message()),
    }
}

pub async fn remove_target_action(
    client: &mut GatehouseClient<Channel>,
    args: TargetCmdAddActionArgs,
) {
    println!("{:?}", args);
    let req = RemoveTargetActionRequest {
        name: args.name,
        typestr: args.typestr,
        actions: args.actions,
    };

    match client.remove_target_action(req).await {
        Ok(response) => {
            let tgt = response.into_inner().target;
            if tgt.is_some() {
                println!("Updated: {}", tgt.unwrap());
            } else {
                println!("Error: No target returned after adding");
            }
        }
        Err(err) => println!("Error: {}", err.message()),
    }
}

pub async fn remove_target(client: &mut GatehouseClient<Channel>, args: TargetCmdRemoveArgs) {
    println!("{:?}", args);
    let req = RemoveTargetRequest {
        name: args.name,
        typestr: args.typestr,
    };

    match client.remove_target(req).await {
        Ok(response) => {
            let tgt = response.into_inner().target;
            if tgt.is_some() {
                println!("Updated: {}", tgt.unwrap());
            } else {
                println!("Error: No target returned after adding");
            }
        }
        Err(err) => println!("Error: {}", err.message()),
    }
}

use tonic::transport::Channel;

use gatehouse::helpers;
use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use crate::args::{TargetCmdAddArgs, TargetCmdModifyArgs, TargetCmdRemoveArgs};

/// convert attributes passed into what the helper expects
fn form_attributes(attr_args: &[String]) -> Vec<(String, Vec<&str>)> {
    let mut attrs = Vec::new();

    for attr_arg in attr_args {
        if let Some((name, vals)) = attr_arg.split_once(':') {
            let values: Vec<&str> = vals.split(',').collect();
            attrs.push((name.to_string(), values))
        }
    }

    attrs
}

pub async fn add_target(client: &mut GatehouseClient<Channel>, args: TargetCmdAddArgs) {
    let attributes = form_attributes(&args.attribs);

    match helpers::add_target(
        client,
        &args.name,
        &args.typestr,
        args.actions.iter().map(AsRef::as_ref).collect(),
        attributes,
    )
    .await
    {
        Ok(target) => println!("Added {target}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

pub async fn modify_target(client: &mut GatehouseClient<Channel>, args: TargetCmdModifyArgs) {
    let add_attributes = form_attributes(&args.add_attribs);
    let remove_attributes = form_attributes(&args.remove_attribs);

    match helpers::modify_target(
        client,
        &args.name,
        &args.typestr,
        args.add_actions.iter().map(AsRef::as_ref).collect(),
        add_attributes,
        args.remove_actions.iter().map(AsRef::as_ref).collect(),
        remove_attributes,
    )
    .await
    {
        Ok(target) => println!("Added {target}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

pub async fn remove_target(client: &mut GatehouseClient<Channel>, args: TargetCmdRemoveArgs) {
    match helpers::remove_target(client, &args.name, &args.typestr).await {
        Ok(target) => println!("Remove {target}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}

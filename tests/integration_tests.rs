mod common;

use std::time::Duration;

use tokio::test;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use crate::common::{
    add_target, add_target_actions, get_targets, remove_target, remove_target_actions,
};

#[test]
async fn test_targets() {
    tokio::spawn(common::run_server());
    tokio::time::sleep(Duration::from_secs(2)).await;

    let mut client = GatehouseClient::connect("http://localhost:6174")
        .await
        .expect("could not create client");

    // ensure we have no targets at the start
    let targets = get_targets(&mut client, None, None).await;

    assert_eq!(targets.len(), 0, "targets should have been 0");

    // add a target
    let tgt1 = add_target(&mut client, "db1", "database", Vec::new()).await;
    assert_eq!(tgt1.name, "db1");
    assert_eq!(tgt1.typestr, "database");
    assert_eq!(tgt1.actions.len(), 0);

    // add some more targets
    let tgt2 = add_target(&mut client, "db2", "database", vec!["read", "write"]).await;
    let tgt3 = add_target(&mut client, "www1", "website", Vec::new()).await;
    let _tgt4 = add_target(&mut client, "www2", "website", Vec::new()).await;
    let _tgt5 = add_target(&mut client, "login", "website", Vec::new()).await;
    assert_eq!(tgt2.actions.len(), 2);

    // make sure all the targets are there
    let targets = get_targets(&mut client, None, None).await;
    assert_eq!(targets.len(), 5, "expected 5 targets");

    // filter targets by type
    let targets = get_targets(&mut client, None, Some("website")).await;
    assert_eq!(
        targets.len(),
        3,
        "expected to get 3 targets of type website"
    );

    // filter target type by name
    let targets = get_targets(&mut client, Some("db2"), None).await;
    assert_eq!(targets.len(), 1, "expected to get a single result");
    assert_eq!(targets[0].name, "db2");

    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 0);
    let tgt3 = add_target_actions(&mut client, "www1", "website", vec!["login", "logout"]).await;
    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 2);

    let tgt3 = remove_target_actions(&mut client, "www1", "website", vec!["logout"]).await;
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");

    let tgt3 = remove_target(&mut client, "www1", "website").await;
    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
}

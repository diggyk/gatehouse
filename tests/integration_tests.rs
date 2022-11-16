mod common;

use std::time::Duration;

use common::get_roles;
use tokio::test;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use crate::common::{
    add_entity, add_role, add_target, get_entities, get_targets, modify_entity, modify_target,
    remove_entity, remove_role, remove_target, str,
};

#[test]
async fn test_crud() {
    tokio::spawn(common::run_server());
    tokio::time::sleep(Duration::from_secs(2)).await;

    test_targets().await;
    test_entities().await;
    test_roles().await;
}

async fn test_targets() {
    let mut client = GatehouseClient::connect("http://localhost:6174")
        .await
        .expect("could not create client");

    // ensure we have no targets at the start
    let targets = get_targets(&mut client, None, None).await;

    assert_eq!(targets.len(), 0, "targets should have been 0");

    // add a target
    let tgt1 = add_target(&mut client, "db1", "database", vec![], vec![]).await;
    assert_eq!(tgt1.name, "db1");
    assert_eq!(tgt1.typestr, "database");
    assert_eq!(tgt1.actions.len(), 0);

    // add some more targets
    let tgt2 = add_target(
        &mut client,
        "db2",
        "database",
        vec!["read", "write"],
        vec![(str("role"), vec!["prod"])],
    )
    .await;
    let tgt3 = add_target(&mut client, "www1", "website", vec![], vec![]).await;
    let _tgt4 = add_target(&mut client, "www2", "website", vec![], vec![]).await;
    let _tgt5 = add_target(&mut client, "login", "website", vec![], vec![]).await;
    assert_eq!(tgt2.actions.len(), 2);
    assert!(tgt2.attributes.contains_key("role"));

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
    let tgt3 = modify_target(
        &mut client,
        "www1",
        "website",
        vec!["login", "logout"],
        vec![],
        vec![],
        vec![],
    )
    .await;
    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 2);

    // remove an action and add some attributes
    let tgt3 = modify_target(
        &mut client,
        "www1",
        "website",
        vec![],
        vec![
            (str("auth"), vec!["basic", "gssapi"]),
            (str("api"), vec!["json", "xml"]),
        ],
        vec!["logout"],
        vec![],
    )
    .await;
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
    assert!(tgt3.attributes.contains_key("auth"));
    assert!(tgt3.attributes.contains_key("api"));
    assert_eq!(
        tgt3.attributes
            .get("auth")
            .expect("Could not find auth attrib")
            .values
            .len(),
        2
    );
    assert_eq!(
        tgt3.attributes
            .get("api")
            .expect("Could not find api attrib")
            .values
            .len(),
        2
    );

    // remove some attributes
    let tgt3 = modify_target(
        &mut client,
        "www1",
        "website",
        vec![],
        vec![],
        vec![],
        vec![(str("api"), vec!["json"])],
    )
    .await;
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
    assert!(tgt3.attributes.contains_key("auth"));
    assert!(tgt3.attributes.contains_key("api"));
    assert_eq!(
        tgt3.attributes
            .get("auth")
            .expect("Could not find auth attrib")
            .values
            .len(),
        2
    );
    assert_eq!(
        tgt3.attributes
            .get("api")
            .expect("Could not find api attrib")
            .values[0],
        "xml"
    );

    // remove the last value for api attributes and verify it is all cleared
    // remove some attributes
    let tgt3 = modify_target(
        &mut client,
        "www1",
        "website",
        vec![],
        vec![],
        vec![],
        vec![(str("api"), vec!["xml"])],
    )
    .await;
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
    assert!(tgt3.attributes.contains_key("auth"));
    assert!(!tgt3.attributes.contains_key("api"));
    assert_eq!(
        tgt3.attributes
            .get("auth")
            .expect("Could not find auth attrib")
            .values
            .len(),
        2
    );

    let tgt3 = remove_target(&mut client, "www1", "website").await;
    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
}

async fn test_entities() {
    let mut client = GatehouseClient::connect("http://localhost:6174")
        .await
        .expect("could not create client");

    // ensure we have no entities at the start
    let entities = get_entities(&mut client, None, None).await;

    assert_eq!(entities.len(), 0, "entities should have been 0");

    // add a entity
    let ent1 = add_entity(&mut client, "testman1", "user", vec![]).await;
    assert_eq!(ent1.name, "testman1");
    assert_eq!(ent1.typestr, "user");
    assert!(ent1.attributes.is_empty());

    // add some more entities
    let ent2 = add_entity(
        &mut client,
        "sandytest",
        "user",
        vec![(str("org"), vec!["hr"])],
    )
    .await;
    let _ = add_entity(&mut client, "logger", "svc", vec![]).await;
    let _ = add_entity(&mut client, "launcher", "svc", vec![]).await;
    let _ = add_entity(&mut client, "printer", "svc", vec![]).await;
    assert!(ent2.attributes.contains_key("org"));

    // make sure all the entities are there
    let entities = get_entities(&mut client, None, None).await;
    assert_eq!(entities.len(), 5, "expected 5 entities");

    // filter entities by type
    let entities = get_entities(&mut client, None, Some("svc")).await;
    assert_eq!(
        entities.len(),
        3,
        "expected to get 3 entities of type website"
    );

    // filter target type by name
    let entities = get_entities(&mut client, Some("sandytest"), None).await;
    assert_eq!(entities.len(), 1, "expected to get a single result");
    assert_eq!(entities[0].name, "sandytest");

    // add some attributes
    let ent3 = modify_entity(
        &mut client,
        "logger",
        "svc",
        vec![
            (str("org"), vec!["hr", "recruiting"]),
            (str("office"), vec!["remote", "nyc"]),
        ],
        vec![],
    )
    .await;
    assert!(ent3.attributes.contains_key("org"));
    assert!(ent3.attributes.contains_key("office"));
    assert_eq!(
        ent3.attributes
            .get("org")
            .expect("Could not find org attrib")
            .values
            .len(),
        2
    );
    assert_eq!(
        ent3.attributes
            .get("office")
            .expect("Could not find office attrib")
            .values
            .len(),
        2
    );

    // remove some attributes
    let ent3 = modify_entity(
        &mut client,
        "logger",
        "svc",
        vec![],
        vec![(str("office"), vec!["remote"])],
    )
    .await;
    assert!(ent3.attributes.contains_key("org"));
    assert!(ent3.attributes.contains_key("office"));
    assert_eq!(
        ent3.attributes
            .get("org")
            .expect("Could not find org attrib")
            .values
            .len(),
        2
    );
    assert_eq!(
        ent3.attributes
            .get("office")
            .expect("Could not find office attrib")
            .values[0],
        "nyc"
    );

    // remove the last value for api attributes and verify it is all cleared
    // remove some attributes
    let ent3 = modify_entity(
        &mut client,
        "logger",
        "svc",
        vec![],
        vec![(str("office"), vec!["nyc"])],
    )
    .await;
    assert!(ent3.attributes.contains_key("org"));
    assert!(!ent3.attributes.contains_key("office"));
    assert_eq!(
        ent3.attributes
            .get("org")
            .expect("Could not find auth attrib")
            .values
            .len(),
        2
    );

    let ent3 = remove_entity(&mut client, "logger", "svc").await;
    assert_eq!(ent3.name, "logger");
    assert_eq!(ent3.typestr, "svc");
}

async fn test_roles() {
    let mut client = GatehouseClient::connect("http://localhost:6174")
        .await
        .expect("could not create client");

    let roles = get_roles(&mut client, None).await;
    assert_eq!(roles.len(), 0, "expected 0 roles");

    let role1 = add_role(&mut client, "power-admin").await;
    assert_eq!(role1.name, "power-admin");

    let _ = add_role(&mut client, "launch-guard").await;
    let _ = add_role(&mut client, "auditer").await;

    let roles = get_roles(&mut client, None).await;
    assert_eq!(roles.len(), 3, "expected 3 roles");

    let role = remove_role(&mut client, "launch-guard").await;
    assert_eq!(role.name, "launch-guard");

    let roles = get_roles(&mut client, None).await;
    assert_eq!(roles.len(), 2, "expected 2 roles");
}

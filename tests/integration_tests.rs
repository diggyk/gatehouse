mod common;

use std::process::exit;

use tokio::time::{sleep, Duration};

use gatehouse::proto::policies::{
    Decide, EntityCheck, KvCheck, Num, NumberCheck, Set, StringCheck, TargetCheck,
};
use tokio::test;

use gatehouse::proto::base::gatehouse_client::GatehouseClient;

use gatehouse::helpers::{
    add_entity, add_group, add_policy, add_role, add_target, get_entities, get_groups,
    get_policies, get_roles, get_targets, modify_entity, modify_group, modify_policy,
    modify_target, remove_entity, remove_group, remove_policy, remove_role, remove_target, str,
};
use tonic::transport::Channel;

#[test]
async fn test_crud() {
    let handle = tokio::spawn(common::run_server());
    sleep(Duration::from_secs(5)).await;

    test_targets().await;
    test_entities().await;
    test_roles().await;
    test_groups().await;
    test_polices().await;

    handle.abort();
    sleep(Duration::from_secs(5)).await;

    tokio::spawn(common::run_server());
    sleep(Duration::from_secs(5)).await;

    load_data().await;
}

async fn create_client() -> GatehouseClient<Channel> {
    let mut attempts = 0;
    loop {
        match GatehouseClient::connect("http://localhost:6174").await {
            Ok(client) => return client,
            Err(err) => {
                attempts += 1;

                if attempts > 5 {
                    println!("Could not create client: {err}");
                    exit(1);
                }
            }
        }
    }
}

async fn test_targets() {
    let mut client = create_client().await;

    // ensure we have no targets at the start
    let targets = get_targets(&mut client, None, None).await.unwrap();

    assert_eq!(targets.len(), 0, "targets should have been 0");

    // add a target
    let tgt1 = add_target(&mut client, "db1", "database", vec![], vec![])
        .await
        .expect("Didn't get target");
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
    .await
    .expect("Didn't get target");
    let tgt3 = add_target(&mut client, "www1", "website", vec![], vec![])
        .await
        .expect("Didn't get target");
    let _tgt4 = add_target(&mut client, "www2", "website", vec![], vec![])
        .await
        .expect("Didn't get target");
    let _tgt5 = add_target(&mut client, "login", "website", vec![], vec![])
        .await
        .expect("Didn't get target");
    assert_eq!(tgt2.actions.len(), 2);
    assert!(tgt2.attributes.contains_key("role"));

    // make sure all the targets are there
    let targets = get_targets(&mut client, None, None).await.unwrap();
    assert_eq!(targets.len(), 5, "expected 5 targets");

    // filter targets by type
    let targets = get_targets(&mut client, None, Some("website"))
        .await
        .unwrap();
    assert_eq!(
        targets.len(),
        3,
        "expected to get 3 targets of type website"
    );

    // filter target type by name
    let targets = get_targets(&mut client, Some("db2"), None).await.unwrap();
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
    .await
    .unwrap();
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
    .await
    .unwrap();
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
    .await
    .unwrap();
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
    .await
    .unwrap();
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

    let tgt3 = remove_target(&mut client, "www1", "website").await.unwrap();
    assert_eq!(tgt3.name, "www1");
    assert_eq!(tgt3.typestr, "website");
    assert_eq!(tgt3.actions.len(), 1);
    assert_eq!(tgt3.actions[0], "login");
}

async fn test_entities() {
    let mut client = create_client().await;

    // ensure we have no entities at the start
    let entities = get_entities(&mut client, None, None).await.unwrap();

    assert_eq!(entities.len(), 0, "entities should have been 0");

    // add a entity
    let ent1 = add_entity(&mut client, "testman1", "user", vec![])
        .await
        .unwrap();
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
    .await
    .unwrap();
    let _ = add_entity(&mut client, "logger", "svc", vec![])
        .await
        .unwrap();
    let _ = add_entity(&mut client, "launcher", "svc", vec![])
        .await
        .unwrap();
    let _ = add_entity(&mut client, "printer", "svc", vec![])
        .await
        .unwrap();
    assert!(ent2.attributes.contains_key("org"));

    // make sure all the entities are there
    let entities = get_entities(&mut client, None, None).await.unwrap();
    assert_eq!(entities.len(), 5, "expected 5 entities");

    // filter entities by type
    let entities = get_entities(&mut client, None, Some("svc")).await.unwrap();
    assert_eq!(
        entities.len(),
        3,
        "expected to get 3 entities of type website"
    );

    // filter target type by name
    let entities = get_entities(&mut client, Some("sandytest"), None)
        .await
        .unwrap();
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
    .await
    .unwrap();
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
    .await
    .unwrap();
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
    .await
    .unwrap();
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

    let ent3 = remove_entity(&mut client, "logger", "svc").await.unwrap();
    assert_eq!(ent3.name, "logger");
    assert_eq!(ent3.typestr, "svc");
}

async fn test_roles() {
    let mut client = create_client().await;

    let roles = get_roles(&mut client, None).await.unwrap();
    assert_eq!(roles.len(), 0, "expected 0 roles");

    let role1 = add_role(&mut client, "power-admin").await.unwrap();
    assert_eq!(role1.name, "power-admin");
    assert_eq!(role1.granted_to.len(), 0);

    let _ = add_role(&mut client, "launch-guard").await.unwrap();
    let _ = add_role(&mut client, "auditer").await.unwrap();

    let roles = get_roles(&mut client, None).await.unwrap();
    assert_eq!(roles.len(), 3, "expected 3 roles");

    let role = remove_role(&mut client, "launch-guard").await.unwrap();
    assert_eq!(role.name, "launch-guard");

    let roles = get_roles(&mut client, None).await.unwrap();
    assert_eq!(roles.len(), 2, "expected 2 roles");
}

async fn test_groups() {
    let mut client = create_client().await;

    let role1 = add_role(&mut client, "admin").await.unwrap();
    let role2 = add_role(&mut client, "user").await.unwrap();
    let role3 = add_role(&mut client, "guest").await.unwrap();
    let role4 = add_role(&mut client, "manager").await.unwrap();
    assert_eq!(role1.name, "admin");
    assert_eq!(role2.name, "user");
    assert_eq!(role3.name, "guest");
    assert_eq!(role4.name, "manager");

    let grp1 = add_group(
        &mut client,
        "administrators",
        None,
        vec![("sandytest", "authuser"), ("donnyman", "authuser")],
        vec!["admin", "user"],
    )
    .await
    .unwrap();

    assert_eq!(grp1.name, "administrators");
    assert_eq!(grp1.members.len(), 2);
    assert_eq!(grp1.roles.len(), 2);

    let role1 = get_roles(&mut client, Some("admin")).await.unwrap();
    assert_eq!(role1[0].name, "admin");
    assert_eq!(role1[0].granted_to.len(), 1);
    assert_eq!(role1[0].granted_to[0], "administrators");
    let role2 = get_roles(&mut client, Some("user")).await.unwrap();
    assert_eq!(role2[0].name, "user");
    assert_eq!(role2[0].granted_to.len(), 1);
    assert_eq!(role2[0].granted_to[0], "administrators");

    let grp1 = modify_group(
        &mut client,
        "administrators",
        None,
        vec![("testman", "authuser"), ("testdog", "authuser")],
        vec!["manager", "guest"],
        vec![("sandytest", "authuser")],
        vec!["admin"],
    )
    .await
    .unwrap();

    let grp2 = add_group(
        &mut client,
        "customers",
        None,
        vec![("coke", "authuser"), ("pepsi", "authuser")],
        vec!["user", "manager"],
    )
    .await
    .unwrap();

    assert_eq!(grp1.name, "administrators");
    assert_eq!(grp1.members.len(), 3);
    assert_eq!(grp1.roles.len(), 3);

    assert_eq!(grp2.name, "customers");
    assert_eq!(grp2.members.len(), 2);
    assert_eq!(grp2.roles.len(), 2);

    let role1 = get_roles(&mut client, Some("admin")).await.unwrap();
    assert_eq!(role1[0].name, "admin");
    assert_eq!(role1[0].granted_to.len(), 0);
    let role2 = get_roles(&mut client, Some("user")).await.unwrap();
    assert_eq!(role2[0].name, "user");
    assert_eq!(role2[0].granted_to.len(), 2);

    let grp1 = remove_group(&mut client, "administrators").await.unwrap();
    assert_eq!(grp1.name, "administrators");
    let role1 = get_roles(&mut client, Some("admin")).await.unwrap();
    assert_eq!(role1[0].name, "admin");
    assert_eq!(role1[0].granted_to.len(), 0);
    let role2 = get_roles(&mut client, Some("user")).await.unwrap();
    assert_eq!(role2[0].name, "user");
    assert_eq!(role2[0].granted_to.len(), 1);

    // we will next remove a role and make sure it gets removed from the group
    let grp2 = get_groups(&mut client, Some("customers"), None, None)
        .await
        .unwrap();
    assert_eq!(grp2[0].name, "customers");
    assert_eq!(grp2[0].roles.len(), 2);
    assert!(grp2[0].roles.iter().any(|r| r == "user"));

    let role2 = remove_role(&mut client, "user").await.unwrap();
    assert_eq!(role2.name, "user");

    let grp2 = get_groups(&mut client, Some("customers"), None, None)
        .await
        .unwrap();
    assert_eq!(grp2[0].name, "customers");
    assert_eq!(grp2[0].roles.len(), 1);
    assert!(!grp2[0].roles.iter().any(|r| r == "user"));
}

async fn test_polices() {
    let mut client = create_client().await;

    let pol1 = add_policy(
        &mut client,
        "allow-admins",
        None,
        Some(EntityCheck {
            name: None,
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("user")],
            }),
            attributes: vec![KvCheck {
                key: str("role"),
                op: Set::Has.into(),
                vals: vec![str("admin")],
            }],
            bucket: None,
        }),
        vec![],
        None,
        Decide::Allow,
    )
    .await
    .unwrap();

    assert_eq!(pol1.name, "allow-admins");
    assert_eq!(pol1.decision(), Decide::Allow);
    assert_eq!(pol1.entity_check.unwrap_or_default().bucket, None);

    let pol1 = modify_policy(
        &mut client,
        "allow-admins",
        None,
        Some(EntityCheck {
            name: None,
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("user")],
            }),
            attributes: vec![KvCheck {
                key: str("role"),
                op: Set::Has.into(),
                vals: vec![str("admin")],
            }],
            bucket: Some(NumberCheck {
                op: Num::LessThan.into(),
                val: 50,
            }),
        }),
        vec![],
        None,
        Decide::Deny,
    )
    .await
    .unwrap();

    assert_eq!(pol1.name, "allow-admins");
    assert_eq!(pol1.decision(), Decide::Deny);
    assert_eq!(
        pol1.clone()
            .entity_check
            .unwrap_or_default()
            .bucket
            .unwrap_or_default()
            .val,
        50
    );

    let _ = add_policy(
        &mut client,
        "allow-everyone",
        None,
        None,
        vec![],
        None,
        Decide::Allow,
    )
    .await
    .unwrap();

    let pols = get_policies(&mut client, None).await.unwrap();
    assert_eq!(pols.len(), 2);

    let pol_found = get_policies(&mut client, Some("allow-admins"))
        .await
        .unwrap();
    assert_eq!(pol_found.len(), 1);
    assert_eq!(pol1, pol_found[0]);

    let pol_removed = remove_policy(&mut client, "allow-admins").await.unwrap();
    assert_eq!(pol1, pol_removed);

    let pols = get_policies(&mut client, None).await.unwrap();
    assert_eq!(pols.len(), 1);
    assert_eq!(pols[0].name, "allow-everyone");

    let _ = add_policy(
        &mut client,
        "allow-teamalpha",
        Some("Give specific access to members of Team Alpha"),
        Some(EntityCheck {
            name: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("brandy"), str("hank")],
            }),
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("user")],
            }),
            attributes: vec![
                KvCheck {
                    key: str("role"),
                    op: Set::Has.into(),
                    vals: vec![str("admin")],
                },
                KvCheck {
                    key: str("role"),
                    op: Set::HasNot.into(),
                    vals: vec![str("manager"), str("exec")],
                },
            ],
            bucket: Some(NumberCheck {
                op: Num::LessThan.into(),
                val: 50,
            }),
        }),
        vec![KvCheck {
            key: str("env"),
            op: Set::Has.into(),
            vals: vec![str("prod")],
        }],
        Some(TargetCheck {
            name: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("launchctl"), str("abortctl")],
            }),
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("svc"), str("api"), str("ui")],
            }),
            attributes: vec![KvCheck {
                key: str("release"),
                op: Set::Has.into(),
                vals: vec![str("stable"), str("canary")],
            }],
            action: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("engage"), str("check"), str("read")],
            }),
            match_in_entity: vec![],
            match_in_env: vec![],
        }),
        Decide::Allow,
    )
    .await
    .unwrap();

    let pol_found = get_policies(&mut client, None).await.unwrap();
    assert_eq!(pol_found.len(), 2);
}

async fn load_data() {
    let mut client = create_client().await;

    add_target(
        &mut client,
        "db1",
        "database",
        vec!["read", "write", "update", "delete"],
        vec![(str("role"), vec!["master"]), (str("schema"), vec!["v20"])],
    )
    .await
    .expect("Didn't get target");
    add_target(
        &mut client,
        "db2",
        "database",
        vec!["read", "write", "update", "delete"],
        vec![(str("role"), vec!["replica"]), (str("schema"), vec!["v20"])],
    )
    .await
    .expect("Didn't get target");
    add_target(
        &mut client,
        "www1",
        "website",
        vec!["view-users", "create-users", "read-metrics"],
        vec![(str("region"), vec!["us-west"])],
    )
    .await
    .expect("Didn't get target");
    add_target(
        &mut client,
        "www2",
        "website",
        vec!["view-users", "create-users", "read-metrics"],
        vec![(str("region"), vec!["emea"])],
    )
    .await
    .expect("Didn't get target");
    add_target(
        &mut client,
        "login",
        "website",
        vec!["view-users", "create-users", "read-metrics"],
        vec![(str("region"), vec!["anz"])],
    )
    .await
    .expect("Didn't get target");

    add_entity(
        &mut client,
        "sally",
        "user",
        vec![
            (str("team"), vec!["eng"]),
            (str("office"), vec!["london", "remote"]),
            (str("clearance"), vec!["secret"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "kelsey",
        "user",
        vec![
            (str("team"), vec!["eng"]),
            (str("office"), vec!["nyc"]),
            (str("clearance"), vec!["secret"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "john",
        "user",
        vec![
            (str("team"), vec!["eng"]),
            (str("office"), vec!["sfo"]),
            (str("clearance"), vec!["none"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "devraj",
        "user",
        vec![
            (str("team"), vec!["eng"]),
            (str("office"), vec!["sfo"]),
            (str("clearance"), vec!["none"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "marie",
        "user",
        vec![
            (str("team"), vec!["eng"]),
            (str("office"), vec!["sfo", "remote"]),
            (str("clearance"), vec!["none"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "catie",
        "user",
        vec![
            (str("team"), vec!["ceo", "exec"]),
            (str("office"), vec!["sfo"]),
            (str("clearance"), vec!["secret"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "rose",
        "user",
        vec![
            (str("team"), vec!["exec"]),
            (str("office"), vec!["sfo"]),
            (str("clearance"), vec!["none"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "jack",
        "user",
        vec![
            (str("team"), vec!["ceo"]),
            (str("office"), vec!["nyc", "remote"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "logger",
        "svc",
        vec![(str("env"), vec!["prod"])],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "launcher",
        "svc",
        vec![
            (str("env"), vec!["dev"]),
            (str("clearance"), vec!["secret"]),
        ],
    )
    .await
    .unwrap();
    add_entity(
        &mut client,
        "printer",
        "svc",
        vec![(str("env"), vec!["dev"])],
    )
    .await
    .unwrap();

    add_role(&mut client, "admin").await.unwrap();
    add_role(&mut client, "user").await.unwrap();
    add_role(&mut client, "guest").await.unwrap();
    add_role(&mut client, "manager").await.unwrap();

    add_group(
        &mut client,
        "administrators",
        Some("Administrators with special privileges"),
        vec![
            ("john", "user"),
            ("adminapi", "svc"),
            ("kelsey", "user"),
            ("marie", "user"),
        ],
        vec!["admin", "user"],
    )
    .await
    .unwrap();

    add_group(
        &mut client,
        "customers",
        Some("Beta customers"),
        vec![
            ("coke", "authuser"),
            ("pepsi", "authuser"),
            ("marie", "user"),
        ],
        vec!["user", "guest"],
    )
    .await
    .unwrap();

    add_policy(
        &mut client,
        "allow-everyone",
        None,
        None,
        vec![],
        None,
        Decide::Allow,
    )
    .await
    .unwrap();

    add_policy(
        &mut client,
        "allow-teamalpha",
        Some("Give specific access to members of Team Alpha"),
        Some(EntityCheck {
            name: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("john"), str("kelsey"), str("sally")],
            }),
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("user")],
            }),
            attributes: vec![
                KvCheck {
                    key: str("role"),
                    op: Set::Has.into(),
                    vals: vec![str("admin")],
                },
                KvCheck {
                    key: str("role"),
                    op: Set::HasNot.into(),
                    vals: vec![str("manager"), str("exec")],
                },
            ],
            bucket: Some(NumberCheck {
                op: Num::LessThan.into(),
                val: 50,
            }),
        }),
        vec![KvCheck {
            key: str("env"),
            op: Set::Has.into(),
            vals: vec![str("prod")],
        }],
        Some(TargetCheck {
            name: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("launchctl"), str("abortctl")],
            }),
            typestr: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("svc"), str("api"), str("ui")],
            }),
            attributes: vec![KvCheck {
                key: str("release"),
                op: Set::Has.into(),
                vals: vec![str("stable"), str("canary")],
            }],
            action: Some(StringCheck {
                val_cmp: Set::Has.into(),
                vals: vec![str("engage"), str("check"), str("read")],
            }),
            match_in_entity: vec![str("clearance")],
            match_in_env: vec![str("env")],
        }),
        Decide::Allow,
    )
    .await
    .unwrap();
}

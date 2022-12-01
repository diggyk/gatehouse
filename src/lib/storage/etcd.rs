use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;

use etcd_client::{Client, Event, EventType, GetOptions, WatchOptions, WatchStream};
use flume::Receiver;
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tonic::async_trait;

use crate::actor::RegisteredActor;
use crate::group::RegisteredGroup;
use crate::msgs::DsRequest;
use crate::policy::RegisteredPolicyRule;
use crate::role::RegisteredRole;
use crate::storage::BackendUpdate;
use crate::target::RegisteredTarget;

use super::Storage;

lazy_static! {
    static ref TYPE_MATCH: Regex = Regex::new(r"/gatehouse/(.*?)/(.*)").unwrap();
}

fn econv<T: std::fmt::Display>(err: T) -> String {
    err.to_string()
}

pub(crate) struct EtcdStorage {
    basepath: String,
    client: Client,
}

impl EtcdStorage {
    pub async fn new(url: &str, req_tx: flume::Sender<DsRequest>) -> Self {
        let mut client = match Client::connect([url], None).await {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Could not connect to Etcd storage: {err}");
                exit(1);
            }
        };

        let basepath = String::from("/gatehouse");

        // test our connection to Etcd
        if let Err(err) = client.get(basepath.as_bytes(), None).await {
            eprintln!("Failed to query Etcd on startup: {err}");
            exit(1);
        }

        let last_rev: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));

        match client.get(basepath.as_bytes(), None).await {
            Err(err) => {
                eprintln!("Failed to query Etcd on startup: {err}");
                exit(1);
            }
            Ok(response) => {
                let header = response
                    .header()
                    .expect("Error establishing initial Etcd connection");
                let starting_rev = header.revision();
                println!("Starting rev: {starting_rev}");

                *last_rev.lock().await = starting_rev;
            }
        }

        // TODO -- watch and restart if failed
        let last_rev_arc = last_rev.clone();
        let basepath_copy = basepath.clone();
        let client_copy = client.clone();
        tokio::spawn(async move {
            EtcdStorage::watch_manager(client_copy, &basepath_copy, last_rev_arc, req_tx).await
        });

        Self { basepath, client }
    }

    /// The watch manager establishes the watch on Etcd and reestablishs the watch if connectivity
    /// is broken.
    ///
    /// Args:
    /// * client: Etcd client
    /// * basepath: the key prefix underwhich Gatehouse stores data
    /// * last_rev: the last revision number we processed, and where we start watch on restart
    /// * req_tx: our channel to send updates to the datastore
    async fn watch_manager(
        mut client: Client,
        basepath: &str,
        last_rev: Arc<Mutex<i64>>,
        req_tx: flume::Sender<DsRequest>,
    ) {
        loop {
            println!("STARTING WATCH AT {}", last_rev.lock().await);
            // start the watch
            let (mut watcher, watch_stream) = match client
                .watch(
                    basepath,
                    Some(
                        WatchOptions::new()
                            .with_prefix()
                            .with_progress_notify()
                            .with_start_revision(*last_rev.lock().await),
                    ),
                )
                .await
            {
                Ok((w, s)) => (w, s),
                Err(err) => {
                    eprintln!("Could not establish a watch on Etcd: {err}");
                    eprintln!("Retrying in 2 seconds...");
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
            };

            // start our stream watcher that will listen for new updates from Etcd
            let last_rev_copy = last_rev.clone();
            let (kill_signal, kill_receiver) = flume::bounded(1);
            let req_tx_clone = req_tx.clone();
            let stream_watcher = tokio::spawn(async move {
                match Self::watch_changes(watch_stream, last_rev_copy, kill_receiver, req_tx_clone)
                    .await
                {
                    Ok(_) => println!("stream watched exited normally"),
                    Err(err) => println!("stream watch exited with error: {err}"),
                }
            });

            // this loop askes for progress so we can detect if our stream died
            // and also if the stream watcher died
            loop {
                sleep(Duration::from_secs(1)).await;
                if stream_watcher.is_finished() {
                    println!("Stream watched has exited! Restart watching...");
                    break;
                }
                match watcher.request_progress().await {
                    Ok(()) => (),
                    Err(err) => {
                        println!("Error with progress: {err}");
                        break;
                    }
                }
            }

            // we have exited out ping loop because the watch stream stopped or we got
            // an error sending a request_progress, meaning our stream died. We will
            // try to kill the stream_watcher if it is still running
            if !stream_watcher.is_finished() {
                match kill_signal.send(()) {
                    Err(err) => {
                        eprintln!("Error sending kill signal to stream watcher: {err}");
                        // stream_watcher.abort();
                    }
                    Ok(_) => {
                        println!("Waiting for watch stream to stop so we can restart");
                        let _ = stream_watcher.await;
                    }
                }
            } else {
                println!("Stream watcher is already shut down");
            }

            // take a breather before starting back up
            sleep(Duration::from_secs(10)).await;
        }
    }

    /// Watch the stream for incoming changes
    ///
    /// As new messages arrive, we'll send them to the datastore. If we hit an error, send a signal back
    /// to the watch manager so we can restart everything.  If we get a kill signal, stop and exit.
    ///
    /// When new messages are processed, update the revision number so we can restart a watch as needed
    async fn watch_changes(
        mut stream: WatchStream,
        last_rev: Arc<Mutex<i64>>,
        kill_receiver: Receiver<()>,
        req_tx: flume::Sender<DsRequest>,
    ) -> Result<(), String> {
        // build the message back to the datastore
        fn build_update(
            event_type: &EventType,
            obj_type: &str,
            obj_name: &str,
            val: &str,
        ) -> Result<BackendUpdate, String> {
            match event_type {
                EventType::Put => match obj_type {
                    "actors" => {
                        let obj: RegisteredActor = serde_json::from_str(val).map_err(econv)?;
                        Ok(BackendUpdate::PutActor(obj))
                    }
                    "groups" => {
                        let obj: RegisteredGroup = serde_json::from_str(val).map_err(econv)?;
                        Ok(BackendUpdate::PutGroup(obj))
                    }
                    "policies" => {
                        let obj: RegisteredPolicyRule = serde_json::from_str(val).map_err(econv)?;
                        Ok(BackendUpdate::PutPolicyRule(obj))
                    }
                    "roles" => {
                        let obj: RegisteredRole = serde_json::from_str(val).map_err(econv)?;
                        Ok(BackendUpdate::PutRole(obj))
                    }
                    "targets" => {
                        let obj: RegisteredTarget = serde_json::from_str(val).map_err(econv)?;
                        Ok(BackendUpdate::PutTarget(obj))
                    }
                    _ => Err(format!("Unknown object type: {obj_type}")),
                },
                EventType::Delete => match obj_type {
                    "actors" => {
                        let (typestr, name) = obj_name.split_once('/').ok_or_else(|| {
                            format!("Could not get type and name from actor {obj_name}")
                        })?;
                        Ok(BackendUpdate::DeleteActor(
                            typestr.to_string(),
                            name.to_string(),
                        ))
                    }
                    "groups" => Ok(BackendUpdate::DeleteGroup(obj_name.to_string())),
                    "policies" => Ok(BackendUpdate::DeletePolicyRule(obj_name.to_string())),
                    "roles" => Ok(BackendUpdate::DeleteRole(obj_name.to_string())),
                    "targets" => {
                        let (typestr, name) = obj_name.split_once('/').ok_or_else(|| {
                            format!("Could not get type and name from target {obj_name}")
                        })?;
                        Ok(BackendUpdate::DeleteTarget(
                            typestr.to_string(),
                            name.to_string(),
                        ))
                    }
                    _ => Err(format!("Unknown object type: {obj_type}")),
                },
            }
        }

        // handle a put or delete event
        async fn handle_event(
            event: &Event,
            req_tx: &flume::Sender<DsRequest>,
        ) -> Result<(), String> {
            if event.kv().is_none() {
                // TODO -- this is a weird bug so not sure what to do. Log it?
                return Err("No KV in event".to_string());
            }

            let kv = event.kv().unwrap();
            let key = kv.key_str().map_err(econv)?;
            let val = kv.value_str().map_err(econv)?;

            let caps = TYPE_MATCH
                .captures(key)
                .ok_or_else(|| String::from("Could not determine type/name from key"))?;

            let obj_type = caps
                .get(1)
                .ok_or_else(|| format!("Did not get object type from key: {key}"))?;
            let obj_name = caps
                .get(2)
                .ok_or_else(|| format!("Could not get name from key {key}"))?;

            let update = build_update(
                &event.event_type(),
                obj_type.as_str(),
                obj_name.as_str(),
                val,
            )?;

            req_tx
                .send_async(DsRequest::Update(update))
                .await
                .map_err(econv)
        }

        loop {
            if !kill_receiver.is_empty() || kill_receiver.is_disconnected() {
                break;
            }
            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {}
                response = stream.message() => {
                    if let Err(err) = response {
                        // we need to exit so we can be reestablished
                        return Err(format!("Watch stream closed: {err}"));
                    }

                    if let Ok(Some(mut msg)) = response {
                        if let Some(headers) = msg.take_header() {
                            if *last_rev.lock().await == headers.revision() {
                                // we've already processed this revision so break
                                // the loop.
                                continue;
                            }
                            println!("Got message: {}", headers.revision());
                            *last_rev.lock().await = headers.revision();
                        } else {
                            // TODO -- we need some kind of proper error for this?
                            eprintln!("No headers???");
                        }

                        for event in msg.events() {
                            if let Err(err) = handle_event(event, &req_tx).await {
                                eprintln!("Error handling event {:?}: {}", event, err);
                            }
                        }
                    }
                },
            }
        }
        Err("Watch stream was closed".to_string())
    }
}

#[async_trait]
impl Storage for EtcdStorage {
    async fn save_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!("{}/targets/{}/{}", self.basepath, tgt.typestr, tgt.name);

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(target_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }

    async fn remove_target(&self, tgt: &RegisteredTarget) -> Result<(), String> {
        let target_path = format!("{}/targets/{}/{}", self.basepath, tgt.typestr, tgt.name);

        self.client
            .kv_client()
            .delete(target_path, None)
            .await
            .map_err(econv)?;
        Ok(())
    }

    async fn load_targets(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredTarget>>, String> {
        let targets_path = format!("{}/targets", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(targets_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let target: RegisteredTarget = serde_json::from_str(val).map_err(econv)?;
            let typed_targets = map
                .entry(target.typestr.clone())
                .or_insert_with(HashMap::new);
            typed_targets.insert(target.name.clone(), target);
        }

        Ok(map)
    }
    async fn save_actor(&self, tgt: &RegisteredActor) -> Result<(), String> {
        let actor_path = format!("{}/actors/{}/{}", self.basepath, tgt.typestr, tgt.name);

        let json = serde_json::to_string(&tgt).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(actor_path, json, None)
            .await
            .map_err(econv)?;
        Ok(())
    }
    async fn remove_actor(&self, tgt: &RegisteredActor) -> Result<(), String> {
        let actor_path = format!("{}/actors/{}/{}", self.basepath, tgt.typestr, tgt.name);

        self.client
            .kv_client()
            .delete(actor_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_actors(
        &self,
    ) -> Result<HashMap<String, HashMap<String, RegisteredActor>>, String> {
        let actors_path = format!("{}/actors", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(actors_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let actor: RegisteredActor = serde_json::from_str(val).map_err(econv)?;
            let typed_actors = map
                .entry(actor.typestr.clone())
                .or_insert_with(HashMap::new);
            typed_actors.insert(actor.name.clone(), actor);
        }

        Ok(map)
    }
    async fn save_role(&self, role: &RegisteredRole) -> Result<(), String> {
        let role_path = format!("{}/roles/{}", self.basepath, role.name);

        let json = serde_json::to_string(&role).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(role_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_role(&self, name: &str) -> Result<(), String> {
        let role_path = format!("{}/roles/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(role_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_roles(&self) -> Result<HashMap<String, RegisteredRole>, String> {
        let roles_path = format!("{}/roles", self.basepath);

        let results = self
            .client
            .kv_client()
            .get(roles_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in results.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let role: RegisteredRole = serde_json::from_str(val).map_err(econv)?;
            map.insert(role.name.clone(), role);
        }

        Ok(map)
    }
    async fn save_group(&self, group: &RegisteredGroup) -> Result<(), String> {
        let group_path = format!("{}/groups/{}", self.basepath, group.name);

        let json = serde_json::to_string(&group).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(group_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_group(&self, name: &str) -> Result<(), String> {
        let group_path = format!("{}/groups/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(group_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_groups(&self) -> Result<HashMap<String, RegisteredGroup>, String> {
        let groups_path = format!("{}/groups", self.basepath);

        let response = self
            .client
            .kv_client()
            .get(groups_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in response.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let group: RegisteredGroup = serde_json::from_str(val).map_err(econv)?;
            map.insert(group.name.clone(), group);
        }

        Ok(map)
    }
    async fn save_policy(&self, policy: &RegisteredPolicyRule) -> Result<(), String> {
        let policy_path = format!("{}/policies/{}", self.basepath, policy.name);

        let json = serde_json::to_string(&policy).map_err(|err| err.to_string())?;

        self.client
            .kv_client()
            .put(policy_path, json, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn remove_policy(&self, name: &str) -> Result<(), String> {
        let policy_path = format!("{}/policies/{}", self.basepath, name);

        self.client
            .kv_client()
            .delete(policy_path, None)
            .await
            .map_err(econv)?;

        Ok(())
    }
    async fn load_policies(&self) -> Result<HashMap<String, RegisteredPolicyRule>, String> {
        let policies_path = format!("{}/policies", self.basepath);

        let response = self
            .client
            .kv_client()
            .get(policies_path, Some(GetOptions::new().with_prefix()))
            .await
            .map_err(econv)?;

        let mut map = HashMap::new();
        for kv in response.kvs() {
            let val = std::str::from_utf8(kv.value()).map_err(econv)?;
            let policy: RegisteredPolicyRule = serde_json::from_str(val).map_err(econv)?;
            map.insert(policy.name.clone(), policy);
        }

        Ok(map)
    }
}

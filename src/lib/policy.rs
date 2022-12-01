use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::actor::RegisteredActor;
use crate::proto::policies as protos;

/// A string comparison check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum StringCheck {
    // check if string equals one of these values
    OneOf(Vec<String>),
    // check if string is not equal to one of these values
    NotOneOf(Vec<String>),
}
impl StringCheck {
    // check a string value against this string check
    pub fn check(&self, val: &str) -> bool {
        match self {
            StringCheck::OneOf(check_val) => check_val.iter().any(|v| v == val),
            StringCheck::NotOneOf(check_val) => !check_val.iter().any(|v| v == val),
        }
    }
}

/// A key value check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum KvCheck {
    // check if a particular key has one of the given values
    Has(String, Vec<String>),
    // check if a particular key does not have one of the given values
    HasNot(String, Vec<String>),
}
impl KvCheck {
    // check a map of attrib/vals for a match
    pub fn check(&self, attr_map: &HashMap<String, HashSet<String>>) -> bool {
        match self {
            KvCheck::Has(key, vals) => {
                if !attr_map.contains_key(key) {
                    false
                } else if let Some(attr_vals) = attr_map.get(key) {
                    vals.iter().any(|check_val| attr_vals.contains(check_val))
                } else {
                    false
                }
            }
            KvCheck::HasNot(key, vals) => {
                if !attr_map.contains_key(key) {
                    true
                } else if let Some(attr_vals) = attr_map.get(key) {
                    !vals.iter().any(|check_val| attr_vals.contains(check_val))
                } else {
                    true
                }
            }
        }
    }
}

impl From<protos::KvCheck> for KvCheck {
    fn from(kv: protos::KvCheck) -> Self {
        match kv.op() {
            protos::Set::Has => Self::Has(kv.key, kv.vals),
            protos::Set::HasNot => Self::HasNot(kv.key, kv.vals),
        }
    }
}
impl From<KvCheck> for protos::KvCheck {
    fn from(kv: KvCheck) -> Self {
        match kv {
            KvCheck::Has(key, vals) => Self {
                key,
                op: protos::Set::Has.into(),
                vals,
            },
            KvCheck::HasNot(key, vals) => Self {
                key,
                op: protos::Set::HasNot.into(),
                vals,
            },
        }
    }
}

/// A numerical check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum NumberCheck {
    // check if number equals this value
    Equals(i32),
    // check if number is less than this value
    LessThan(i32),
    // check if number is more than this value
    MoreThan(i32),
}
impl NumberCheck {
    /// check if a number passes this check
    pub fn check(&self, num: i32) -> bool {
        match self {
            NumberCheck::Equals(val) => num == *val,
            NumberCheck::LessThan(val) => num < *val,
            NumberCheck::MoreThan(val) => num > *val,
        }
    }
}

impl From<protos::NumberCheck> for NumberCheck {
    fn from(nc: protos::NumberCheck) -> Self {
        match nc.op() {
            protos::Num::Equals => NumberCheck::Equals(nc.val),
            protos::Num::LessThan => NumberCheck::LessThan(nc.val),
            protos::Num::MoreThan => NumberCheck::MoreThan(nc.val),
        }
    }
}
impl From<NumberCheck> for protos::NumberCheck {
    fn from(nc: NumberCheck) -> Self {
        match nc {
            NumberCheck::Equals(val) => Self {
                op: protos::Num::Equals.into(),
                val,
            },
            NumberCheck::LessThan(val) => Self {
                op: protos::Num::LessThan.into(),
                val,
            },
            NumberCheck::MoreThan(val) => Self {
                op: protos::Num::MoreThan.into(),
                val,
            },
        }
    }
}

/// represents the decision of a rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum Decide {
    // rule fails
    Deny,
    // rule passes
    Allow,
}

/// convert from proto to enum
impl From<protos::Decide> for Decide {
    fn from(d: protos::Decide) -> Self {
        match d {
            protos::Decide::Deny => Self::Deny,
            protos::Decide::Allow => Self::Allow,
        }
    }
}
impl From<Decide> for protos::Decide {
    fn from(d: Decide) -> Self {
        match d {
            Decide::Deny => Self::Deny,
            Decide::Allow => Self::Allow,
        }
    }
}
impl Display for protos::Decide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            protos::Decide::Deny => write!(f, "DENY"),
            protos::Decide::Allow => write!(f, "ALLOW"),
        }
    }
}

/// convert the proto to enum
impl From<protos::StringCheck> for StringCheck {
    fn from(sc: protos::StringCheck) -> Self {
        match sc.val_cmp() {
            protos::Set::Has => Self::OneOf(sc.vals),
            protos::Set::HasNot => Self::NotOneOf(sc.vals),
        }
    }
}
impl From<StringCheck> for protos::StringCheck {
    fn from(sc: StringCheck) -> Self {
        match sc {
            StringCheck::OneOf(vals) => Self {
                val_cmp: protos::Set::Has.into(),
                vals,
            },
            StringCheck::NotOneOf(vals) => Self {
                val_cmp: protos::Set::HasNot.into(),
                vals,
            },
        }
    }
}

/// The actor match check in a rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ActorCheck {
    pub name: Option<StringCheck>,
    pub typestr: Option<StringCheck>,
    pub attributes: Vec<KvCheck>,
    pub bucket: Option<NumberCheck>,
}
impl ActorCheck {
    /// perform a check against a potential actor
    pub fn check(&self, actor: &RegisteredActor) -> bool {
        if let Some(ref name_check) = self.name {
            if !name_check.check(&actor.name) {
                // name does not match
                return false;
            }
        }

        if let Some(ref type_check) = self.typestr {
            if !type_check.check(&actor.typestr) {
                // type does not match
                return false;
            }
        }

        if self.attributes.iter().any(|a| !a.check(&actor.attributes)) {
            return false;
        }

        if let Some(ref bucket_check) = self.bucket {
            if !bucket_check.check(actor.bucket().into()) {
                return false;
            }
        }

        true
    }
}

/// convert the protobuf version to our version
impl From<protos::ActorCheck> for ActorCheck {
    fn from(ec: protos::ActorCheck) -> Self {
        Self {
            name: ec.name.map(StringCheck::from),
            typestr: ec.typestr.map(StringCheck::from),
            attributes: ec.attributes.into_iter().map(KvCheck::from).collect(),
            bucket: ec.bucket.map(NumberCheck::from),
        }
    }
}
impl From<ActorCheck> for protos::ActorCheck {
    fn from(ec: ActorCheck) -> Self {
        Self {
            name: ec.name.map(protos::StringCheck::from),
            typestr: ec.typestr.map(protos::StringCheck::from),
            attributes: ec
                .attributes
                .into_iter()
                .map(protos::KvCheck::from)
                .collect(),
            bucket: ec.bucket.map(protos::NumberCheck::from),
        }
    }
}

/// The check to see if the requested target/action match this policy rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct TargetCheck {
    name: Option<StringCheck>,
    typestr: Option<StringCheck>,
    attributes: Vec<KvCheck>,
    match_in_actor: Vec<String>,
    match_in_env: Vec<String>,
    action: Option<StringCheck>,
}
impl TargetCheck {
    /// see if one of our attribute values exists in another set of attribute values
    fn check_attr_match(
        &self,
        attr_to_check: &str,
        our_map: &HashMap<String, HashSet<String>>,
        other_map: &HashMap<String, HashSet<String>>,
    ) -> bool {
        if !our_map.contains_key(attr_to_check) {
            // the target itself doesn't contain the attribute we are supposed to match with the actor
            return false;
        }

        if !other_map.contains_key(attr_to_check) {
            // the actor doesn't contain the attribute we are supposed to check
            return false;
        }

        let our_attr_vals: Vec<&String> = our_map.get(attr_to_check).unwrap().iter().collect();

        let other_attr_vals: Vec<&String> = other_map.get(attr_to_check).unwrap().iter().collect();

        if !our_attr_vals
            .iter()
            .any(|tav| other_attr_vals.contains(tav))
        {
            // we didn't find a single value in the target's attribute values that were also in the
            // actor's attribute values
            return false;
        }

        true
    }

    /// perform a check against a potential actor
    pub fn check(
        &self,
        target_name: &str,
        target_type: &str,
        target_attributes: &HashMap<String, HashSet<String>>,
        target_action: &str,
        actor_attributes: &HashMap<String, HashSet<String>>,
        env_attributes: &HashMap<String, HashSet<String>>,
    ) -> bool {
        if let Some(ref name_check) = self.name {
            if !name_check.check(target_name) {
                // name does not match
                return false;
            }
        }

        if let Some(ref type_check) = self.typestr {
            if !type_check.check(target_type) {
                // type does not match
                return false;
            }
        }

        if self.attributes.iter().any(|a| !a.check(target_attributes)) {
            // one or more attributes do not match
            return false;
        }

        // make sure every attribute that's supposed to match with the actor does match
        for attr_to_check in &self.match_in_actor {
            if !self.check_attr_match(attr_to_check, target_attributes, actor_attributes) {
                // this attribute did not match between the target and the actor
                return false;
            }
        }

        // make sure every attribute that's supposed to match with the environment does match
        for attr_to_check in &self.match_in_env {
            if !self.check_attr_match(attr_to_check, target_attributes, env_attributes) {
                // this attribute did not match between the target and the environment
                return false;
            }
        }

        if let Some(action_check) = &self.action {
            if !action_check.check(target_action) {
                // action does not match
                return false;
            }
        }

        true
    }
}

impl From<protos::TargetCheck> for TargetCheck {
    fn from(tc: protos::TargetCheck) -> Self {
        Self {
            name: tc.name.map(StringCheck::from),
            typestr: tc.typestr.map(StringCheck::from),
            attributes: tc.attributes.into_iter().map(KvCheck::from).collect(),
            match_in_actor: tc.match_in_actor,
            match_in_env: tc.match_in_env,
            action: tc.action.map(StringCheck::from),
        }
    }
}
impl From<TargetCheck> for protos::TargetCheck {
    fn from(tc: TargetCheck) -> protos::TargetCheck {
        Self {
            name: tc.name.map(protos::StringCheck::from),
            typestr: tc.typestr.map(protos::StringCheck::from),
            attributes: tc
                .attributes
                .into_iter()
                .map(protos::KvCheck::from)
                .collect(),
            match_in_actor: tc.match_in_actor,
            match_in_env: tc.match_in_env,
            action: tc.action.map(protos::StringCheck::from),
        }
    }
}

/// A policy rule registered with Gatehouse
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct RegisteredPolicyRule {
    /// the name of this policy rule
    pub name: String,
    /// the optional human description
    pub desc: Option<String>,

    /// determine if actor matches or should match all actors if None
    pub actor_check: Option<ActorCheck>,

    /// list of environment attributes to check
    pub env_attributes: Vec<KvCheck>,

    /// determine if rule applies to target
    pub target_check: Option<TargetCheck>,

    /// The decision to make if this rule matches
    pub decision: Decide,
}

impl From<protos::PolicyRule> for RegisteredPolicyRule {
    fn from(rule: protos::PolicyRule) -> Self {
        let decision = rule.decision();
        Self {
            name: rule.name,
            desc: rule.desc,
            actor_check: rule.actor_check.map(ActorCheck::from),
            env_attributes: rule.env_attributes.into_iter().map(KvCheck::from).collect(),
            target_check: rule.target_check.map(TargetCheck::from),
            decision: Decide::from(decision),
        }
    }
}

impl From<RegisteredPolicyRule> for protos::PolicyRule {
    fn from(rpr: RegisteredPolicyRule) -> Self {
        Self {
            name: rpr.name,
            desc: rpr.desc,
            actor_check: rpr.actor_check.map(ActorCheck::into),
            env_attributes: rpr.env_attributes.into_iter().map(KvCheck::into).collect(),
            target_check: rpr.target_check.map(TargetCheck::into),
            decision: protos::Decide::from(rpr.decision).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;

    fn str(val: &str) -> String {
        val.to_string()
    }

    #[test]
    fn test_stringcheck() {
        assert!(StringCheck::OneOf(vec![str("testing"), str("test2")]).check("testing"));
        assert!(StringCheck::OneOf(vec![str("testing"), str("test2")]).check("test2"));
        assert!(!StringCheck::OneOf(vec![str("testing"), str("test2")]).check("should fail"));

        assert!(!StringCheck::NotOneOf(vec![str("testing"), str("test2")]).check("testing"));
        assert!(StringCheck::NotOneOf(vec![str("testing"), str("test2")]).check("should pass"));
    }

    #[test]
    fn test_kvcheck() {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        map.insert(
            str("role"),
            HashSet::from_iter(vec![str("admin"), str("user")]),
        );
        map.insert(
            str("region"),
            HashSet::from_iter(vec![str("us"), str("emea")]),
        );

        assert!(KvCheck::Has(str("role"), vec![str("banned"), str("user")]).check(&map));
        assert!(!KvCheck::Has(str("role"), vec![str("manager")]).check(&map));
        assert!(KvCheck::HasNot(str("role"), vec![str("manager")]).check(&map));
        assert!(!KvCheck::Has(str("office"), vec![str("london"), str("dublin")]).check(&map));
        assert!(KvCheck::HasNot(str("region"), vec![str("anz")]).check(&map));
        assert!(KvCheck::HasNot(str("office"), vec![str("london")]).check(&map));
    }

    #[test]
    fn test_numcheck() {
        assert!(NumberCheck::Equals(50).check(50));
        assert!(!NumberCheck::Equals(50).check(100));
        assert!(NumberCheck::LessThan(50).check(40));
        assert!(!NumberCheck::LessThan(50).check(100));
        assert!(NumberCheck::MoreThan(50).check(100));
        assert!(!NumberCheck::MoreThan(50).check(40));
    }

    #[test]
    fn test_actorcheck() {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        map.insert(
            str("role"),
            HashSet::from_iter(vec![str("admin"), str("user")]),
        );
        map.insert(str("region"), HashSet::from_iter(vec![str("us")]));
        let actor = RegisteredActor::new("kaitlyn", "user", map);

        // an "everything passes" check
        assert!(ActorCheck {
            name: None,
            typestr: None,
            attributes: vec![],
            bucket: None,
        }
        .check(&actor));

        // check name
        assert!(ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: None,
            attributes: vec![],
            bucket: None,
        }
        .check(&actor));
        assert!(!ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("jonny")])),
            typestr: None,
            attributes: vec![],
            bucket: None,
        }
        .check(&actor));

        // check typestr
        assert!(ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: Some(StringCheck::OneOf(vec![str("user")])),
            attributes: vec![],
            bucket: None,
        }
        .check(&actor));
        assert!(!ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("kaitlyn")])),
            typestr: Some(StringCheck::NotOneOf(vec![str("user")])),
            attributes: vec![],
            bucket: None,
        }
        .check(&actor));

        // check attributes
        assert!(ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: Some(StringCheck::OneOf(vec![str("user")])),
            attributes: vec![KvCheck::Has(str("region"), vec![str("us")])],
            bucket: None,
        }
        .check(&actor));
        assert!(!ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: Some(StringCheck::OneOf(vec![str("user")])),
            attributes: vec![KvCheck::Has(str("role"), vec![str("manager")])],
            bucket: None,
        }
        .check(&actor));

        // check bucket (which is 28)
        assert!(ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: Some(StringCheck::OneOf(vec![str("user")])),
            attributes: vec![KvCheck::Has(str("region"), vec![str("us")])],
            bucket: Some(NumberCheck::LessThan(50)),
        }
        .check(&actor));
        assert!(!ActorCheck {
            name: Some(StringCheck::OneOf(vec![str("betty"), str("kaitlyn")])),
            typestr: Some(StringCheck::OneOf(vec![str("user")])),
            attributes: vec![KvCheck::Has(str("region"), vec![str("us")])],
            bucket: Some(NumberCheck::MoreThan(50)),
        }
        .check(&actor));
    }

    #[test]
    fn test_targetcheck() {
        let mut target_attrs: HashMap<String, HashSet<String>> = HashMap::new();
        target_attrs.insert(
            str("role"),
            HashSet::from_iter(vec![str("main"), str("backup")]),
        );
        target_attrs.insert(str("env"), HashSet::from_iter(vec![str("test")]));

        let mut actor_attrs: HashMap<String, HashSet<String>> = HashMap::new();
        actor_attrs.insert(
            str("office"),
            HashSet::from_iter(vec![str("sfo"), str("remote")]),
        );
        actor_attrs.insert(
            str("env"),
            HashSet::from_iter(vec![str("test"), str("prod")]),
        );

        let mut env_attrs: HashMap<String, HashSet<String>> = HashMap::new();
        env_attrs.insert(str("env"), HashSet::from_iter(vec![str("test")]));

        // test "any target should pass" check
        assert!(TargetCheck {
            name: None,
            typestr: None,
            attributes: vec![],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));

        // test name
        assert!(TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: None,
            attributes: vec![],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
        assert!(!TargetCheck {
            name: Some(StringCheck::NotOneOf(vec![str("bree")])),
            typestr: None,
            attributes: vec![],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));

        // test type
        assert!(TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
        assert!(!TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("web")])),
            attributes: vec![],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));

        // test attributes
        assert!(TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("env"), vec![str("test")])],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
        assert!(!TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("load"), vec![str("nominal")])],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: None,
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));

        // test target action
        assert!(TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("env"), vec![str("test")])],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: Some(StringCheck::OneOf(vec![str("read"),])),
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
        assert!(!TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("env"), vec![str("test")])],
            match_in_actor: vec![],
            match_in_env: vec![],
            action: Some(StringCheck::OneOf(vec![str("write")])),
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));

        // test match_in_actor
        assert!(TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("env"), vec![str("test")])],
            match_in_actor: vec![str("env")],
            match_in_env: vec![],
            action: Some(StringCheck::OneOf(vec![str("read"),])),
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
        assert!(!TargetCheck {
            name: Some(StringCheck::OneOf(vec![str("bree")])),
            typestr: Some(StringCheck::OneOf(vec![str("db")])),
            attributes: vec![KvCheck::Has(str("env"), vec![str("test")])],
            match_in_actor: vec![str("role")],
            match_in_env: vec![],
            action: Some(StringCheck::OneOf(vec![str("read"),])),
        }
        .check(
            "bree",
            "db",
            &target_attrs,
            "read",
            &actor_attrs,
            &env_attrs
        ));
    }
}

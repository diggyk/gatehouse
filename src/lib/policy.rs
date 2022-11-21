use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::entity::RegisteredEntity;
use crate::proto::policies as protos;

/// A string comparison check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum StringCheck {
    // check if string equals this string
    Is(String),
    // chec, if string does not equal this string
    IsNot(String),
}
impl StringCheck {
    // check a string value against this string check
    pub fn check(&self, val: &str) -> bool {
        match self {
            StringCheck::Is(check_val) => val == check_val,
            StringCheck::IsNot(check_val) => val != check_val,
        }
    }
}

/// A key value check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum KvCheck {
    // check if a particular key has a particular value
    Has(String, String),
    // check if a particular key does not have a particular value
    HasNot(String, String),
}
impl KvCheck {
    // check a map of attrib/vals for a match
    pub fn check(&self, map: &HashMap<String, HashSet<String>>) -> bool {
        match self {
            KvCheck::Has(key, val) => {
                if !map.contains_key(key) {
                    false
                } else if let Some(vals) = map.get(key) {
                    vals.contains(val)
                } else {
                    false
                }
            }
            KvCheck::HasNot(key, val) => {
                if !map.contains_key(key) {
                    true
                } else if let Some(vals) = map.get(key) {
                    !vals.contains(val)
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
            protos::Set::Has => Self::Has(kv.key, kv.val),
            protos::Set::HasNot => Self::HasNot(kv.key, kv.val),
        }
    }
}
impl From<KvCheck> for protos::KvCheck {
    fn from(kv: KvCheck) -> Self {
        match kv {
            KvCheck::Has(key, val) => Self {
                key,
                op: protos::Set::Has.into(),
                val,
            },
            KvCheck::HasNot(key, val) => Self {
                key,
                op: protos::Set::HasNot.into(),
                val,
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
    Fail,
    // rule passes
    Pass,
}

/// convert from proto to enum
impl From<protos::Decide> for Decide {
    fn from(d: protos::Decide) -> Self {
        match d {
            protos::Decide::Fail => Self::Fail,
            protos::Decide::Pass => Self::Pass,
        }
    }
}
impl From<Decide> for protos::Decide {
    fn from(d: Decide) -> Self {
        match d {
            Decide::Fail => Self::Fail,
            Decide::Pass => Self::Pass,
        }
    }
}
impl Display for protos::Decide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            protos::Decide::Fail => write!(f, "FAIL"),
            protos::Decide::Pass => write!(f, "PASS"),
        }
    }
}

/// convert the proto to enum
impl From<protos::StringCheck> for StringCheck {
    fn from(sc: protos::StringCheck) -> Self {
        match sc.val_cmp() {
            protos::Cmp::Is => Self::Is(sc.val),
            protos::Cmp::IsNot => Self::IsNot(sc.val),
        }
    }
}
impl From<StringCheck> for protos::StringCheck {
    fn from(sc: StringCheck) -> Self {
        match sc {
            StringCheck::Is(val) => Self {
                val_cmp: protos::Cmp::Is.into(),
                val,
            },
            StringCheck::IsNot(val) => Self {
                val_cmp: protos::Cmp::IsNot.into(),
                val,
            },
        }
    }
}

/// The entity match check in a rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct EntityCheck {
    pub name: Option<StringCheck>,
    pub typestr: Option<StringCheck>,
    pub attributes: Vec<KvCheck>,
    pub bucket: Option<NumberCheck>,
}
impl EntityCheck {
    /// perform a check against a potential entity
    pub fn check(&self, entity: &RegisteredEntity) -> bool {
        if let Some(ref name_check) = self.name {
            if !name_check.check(&entity.name) {
                // name does not match
                return false;
            }
        }

        if let Some(ref type_check) = self.typestr {
            if !type_check.check(&entity.typestr) {
                // type does not match
                return false;
            }
        }

        if self.attributes.iter().any(|a| !a.check(&entity.attributes)) {
            return false;
        }

        if let Some(ref bucket_check) = self.bucket {
            if !bucket_check.check(entity.bucket().into()) {
                return false;
            }
        }

        true
    }
}

/// convert the protobuf version to our version
impl From<protos::EntityCheck> for EntityCheck {
    fn from(ec: protos::EntityCheck) -> Self {
        Self {
            name: ec.name.map(StringCheck::from),
            typestr: ec.typestr.map(StringCheck::from),
            attributes: ec.attributes.into_iter().map(KvCheck::from).collect(),
            bucket: ec.bucket.map(NumberCheck::from),
        }
    }
}
impl From<EntityCheck> for protos::EntityCheck {
    fn from(ec: EntityCheck) -> Self {
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
    action: Option<StringCheck>,
}
impl TargetCheck {
    /// perform a check against a potential entity
    pub fn check(
        &self,
        target_name: &str,
        target_type: &str,
        target_attributes: &HashMap<String, HashSet<String>>,
        target_action: &str,
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

    /// determine if entity matches or should match all entities if None
    pub entity_check: Option<EntityCheck>,

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
            entity_check: rule.entity_check.map(EntityCheck::from),
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
            entity_check: rpr.entity_check.map(EntityCheck::into),
            env_attributes: rpr.env_attributes.into_iter().map(KvCheck::into).collect(),
            target_check: rpr.target_check.map(TargetCheck::into),
            decision: protos::Decide::from(rpr.decision).into(),
        }
    }
}

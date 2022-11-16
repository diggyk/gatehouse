# Gatehouse
Gatehouse is a combination of role-based access control, attribute-based access control, and feature flagging.

# How it works

A Policy Enforcement Point (PEP) is responsible for authenticating the user and then asking Gatehouse if the user can perform a certain action. Gatehouse will trust the PEP to accurately express the user entity and environment properties.

Gatehouse is the Policy Decision Point. It will receive the Entity, Entity Properties, and Environment Properties, along with an Action on a Target. Gatehouse will consult an Entity Information Point to retrieve additional properties that pertain to the Entity. It will then make a policy decision and return that decision to the PEP.

Gatehouse will also have an internal Entity Information Point to map entities to groups and roles. These roles become expressed as attributes on the entity, allowing policy decisions to be made based on them. This capability bridges the RBAC concept to ABAC.

# Gatehouse Primitives

## Targets

A `target` in Gatehouse is composed of a `name`, `type`, and `attributes`. A `target` can have multiple `actions` associated with it. Each of these properties are opaque to Gatehouse: it does not care what the are but they will be used when evaluation policies to determine whether the `entity` passes or fails the access check.

For instance, you might have target types for various websites, infrastructure services, or even a target type for feature flags. The targets might have additional attributes such as environment (e.g. prod, dev, qa) and associated actions such as "read, "write," "admin," etc.

## Entities

An `entity` is asserted by the policy enforcement point to describe the actor wanting to perform an `action` on a `target`. And `entity` is composed of a `name`, `type`, and an optional list of `attributes`.

An `entity` need not be registered with Gatehouse ahead of time, but Gatehouse can act as a lightweight Entity Information Point by appending additional attributes that have been stored for the `entity` before making `policy` decisions.

## Groups

A `group` is composed of a `name`, `entity` members, and a list of `roles`. The `entity` members in a `group` do not need to be registered in Gatehouse as Gatehouse does not insist on being the source of record for entities.

During `policy` evaluation, a `member-of` attribute will be appended to the `entity` for each `group` they belong to before evaluation begins.

## Roles

A `role` in Gatehouse is represented by a single `name` and can be assigned to `groups`. During `policy` evaluation, a `has-role` attribute will be appended to the `entity` for each `role` they have assumed via `group` memberships.

# Policies

A `policy` looks at the totality of properties for the `entity`, their `environment`, and the `target` they wish to act on and makes an `ALLOW` or `DENY` decision.

Along with the `entity` `attributes` supplied by the PEP and appended by Gatehouse from known entities, Gatehouse also automagically creates a `bucket` value between `0` and `99` to allow random distribution when using Gatehouse for feature flags.

**Entity check:**

* name is/isn't
* type is/isn't
* attribute has/hasn't value
* bucket more/equal/less then value

**Environment check:**
* attribute has/hasn't value

**Target check:**
* name is/isn't
* type is/isn't
* attribute has/hasn't value
* action is/isn't/any

## How a policy check works

The Policy Enforcement Point will send a request to Gatehouse asking for an `ALLOW/DENY` decision. This request will be composed of an `entity`, `environment`, and `target` plus `action`. 

Gatehouse will then augment the `attributes` of the `entity` based on any `group` and `role` information in Gatehouse (and external Entity Inforcement Points in the future). Gatehouse will also load the `attributes` for the `target` from its own datastore.

Gatehouse then evaluates this request against the known policies for that `target` and then decides on `ALLOW` or `DENY` based on the following:
* if no matching policy is found, an implicit `DENY` is determined
* if a matching `ALLOW` policy is found, then the decision will to be `ALLOW` unless...
* if an explicit `DENY` policy is found, then the result will always be `DENY`
  

# MVP ToDos

- [x] CRUD Target and Actions
- [ ] CRUD Entities
- [ ] CRUD Groups and Group membership
- [ ] CRUD Roles
- [ ] CRUD Policies
- [ ] Policy engine

# Additional ToDos

- [ ] Tracing
- [ ] Metrics
- [ ] Etcd backend
- [ ] Db backend
- [ ] External information point (LDAP)
- [ ] External information point (DB)

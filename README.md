# Gatehouse
Gatehouse is a combination of role-based access control, attribute-based access control, and feature flagging.

# How it works

A Policy Enforcement Point (PEP) is responsible for authenticating the user and then asking Gatehouse if the user can perform a certain action. Gatehouse will trust the PEP to accurately express the user actor and environment properties.

Gatehouse is the Policy Decision Point. It will receive the Actor, Actor Properties, and Environment Properties, along with an Action on a Target. Gatehouse will consult an Actor Information Point to retrieve additional properties that pertain to the Actor. It will then make a policy decision and return that decision to the PEP.

Gatehouse will also have an internal Actor Information Point to map actors to groups and roles. These roles become expressed as attributes on the actor, allowing policy decisions to be made based on them. This capability bridges the RBAC concept to ABAC.

# Gatehouse Primitives

## Targets

A `target` in Gatehouse is composed of a `name`, `type`, and `attributes`. A `target` can have multiple `actions` associated with it. Each of these properties are opaque to Gatehouse: it does not care what they are but they will be used when evaluating policies to determine whether the `actor` passes or fails the access check.

For instance, you might have target types for various websites, infrastructure services, or even a target type for feature flags. The targets might have additional attributes such as environment (e.g. prod, dev, qa) and associated actions such as "read, "write," "admin," etc.

## Actors

An `actor` is asserted by the policy enforcement point to describe the actor wanting to perform an `action` on a `target`. And `actor` is composed of a `name`, `type`, and an optional list of `attributes`.

An `actor` need not be registered with Gatehouse ahead of time, but Gatehouse can act as a lightweight Actor Information Point by appending additional attributes that have been stored for the `actor` before making `policy` decisions.

## Groups

A `group` is composed of a `name`, `actor` members, and a list of `roles`. The `actor` members in a `group` do not need to be registered in Gatehouse as Gatehouse does not insist on being the source of record for actors.

During `policy` evaluation, a `member-of` attribute will be appended to the `actor` for each `group` they belong to before evaluation begins.

## Roles

A `role` in Gatehouse is represented by a single `name` and can be assigned to `groups`. During `policy` evaluation, a `has-role` attribute will be appended to the `actor` for each `role` they have assumed via `group` memberships.

# Policies

A `policy` looks at the totality of properties for the `actor`, their `environment`, and the `target` they wish to act on and makes an `ALLOW` or `DENY` decision.

Along with the `actor` `attributes` supplied by the PEP and appended by Gatehouse from known actors, Gatehouse also automagically creates a `bucket` value between `0` and `99` to allow random distribution when using Gatehouse for feature flags.

**Actor check:**

* name is/isn't in list of values
* type is/isn't in list of values
* attribute has/hasn't one of a list of values
* bucket more/equal/less then value

**Environment check:**
* attribute has/hasn't one of a list of values

**Target check:**
* name is/isn't in a list of values
* type is/isn't in a list of values
* attribute has/hasn't one of a list of values
* action is/isn't/any

## How a policy check works

The Policy Enforcement Point will send a request to Gatehouse asking for an `ALLOW/DENY` decision. This request will be composed of an `actor`, `environment`, and `target` plus `action`. 

Gatehouse will then augment the `attributes` of the `actor` based on any `group` and `role` information in Gatehouse (and external Actor Inforcement Points in the future). Gatehouse will also load the `attributes` for the `target` from its own datastore.

Gatehouse then evaluates this request against the known policies for that `target` and then decides on `ALLOW` or `DENY` based on the following:
* if no matching policy is found, an implicit `DENY` is determined
* if a matching `ALLOW` policy is found, then the decision will to be `ALLOW` unless...
* if an explicit `DENY` policy is found, then the result will always be `DENY`

# Running Gatehouse

You can run `gatesrv` in a typical Linux environment.  By default, it will store data in `/tmp/gatehouse`
To specify a different backend for storage, set `GATESTORAGE` environment variable to one of the following:

* `file:{path}` store data on the filesystem at the given path
* `etcd:{url}` store data in Etcd by connecting to the given URL
  

# MVP ToDos

- [x] CRUD Target and Actions
- [x] CRUD Actors
- [x] CRUD Groups and Group membership
- [x] CRUD Roles
- [x] CRUD Policies
- [x] Policy engine
- [x] Unit tests
- [x] Integration test
- [x] "MATCH_IN_ENV" and "MATCH_IN_ENT" matchers for Targets
- [ ] gRPC validation
- [ ] Overview stats RPC for total rules/etc

# Additional ToDos

- [ ] Tracing
- [ ] Metrics
- [x] Etcd backend
- [x] Etcd watch for supporting multiserver deployment
- [ ] Db backend
- [ ] External information point (LDAP)
- [ ] External information point (DB)

# gatehouse
RBAC, ABAC, feature flagging

# Early concepts

Policy Enforcement Point (PEP) is responsible for authenticating the user. Gatehouse will trust the PEP to accurately express the entity and environment properties.

Gatehouse is the Policy Decision Point. It will receive the Entity, Entity Properties, and Environment Properties, along with an Action on a Target. Gatehouse will consult an Entity Information Point to retrieve additional properties that pertain to the Entity. It will then make a policy decision and return that decision to the PEP.

Gatehouse will also have an internal Policy Information Point to map entities to groups and roles. These roles become expressed as attributes on the entity, allowing policy decisions to be made based on them. This capability bridges the RBAC concept to ABAC.

# MVP ToDos

- [x] CRUD Target and Actions
- [ ] CRUD Entities
- [ ] CRUD Groups and Group membership
- [ ] CRUD roles
- [ ] CRUD Policies

# Additional ToDos

- [ ] Tracing
- [ ] Metrics
- [ ] Etcd backend
- [ ] Db backend

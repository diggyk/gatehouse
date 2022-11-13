# gatehouse
RBAC, ABAC, feature flagging

# Early concepts

Policy Enforcement Point (PEP) is responsible for authenticating the user. Gatehouse will trust the PEP to accurately express the entity and environment properties.

Gatehouse is the Policy Decision Point. It will receive the Entity, Entity Properties, and Environment Properties, along with an Action on a Target. Gatehouse will consult an Entity Information Point to retrieve additional properties that pertain to the Entity. It will then make a policy decision and return that decision to the PEP.

# submap

B-tree map for pub/sub services.

## Subscription map

### Usage

```rust
let mut smap: SubMap<Client> = SubMap::new();
```

where "Client" is a pub/sub client type, which is usually either a channel or a
structure which contains a channel or locked socket or anything else, required
to work with the client.

The client type MUST provide traits Hash, Eq and Clone.

All clients must be registered in the map, before they can
subscribe/unsubscribe. Use "register\_client" function for this.

When "unregister\_client" is called, it also automatically unsubscribes the
client from all the subscribed topics.

### Separators and wildcards

SubMap supports the following masks:

* this/is/a/topic - single topic subscription
* this/?/a/topic - all topics which match the pattern (2nd chunk - any value)
* this/is/\* - all subtopics of "this/is"
* \* - all topics

Service symbols can be changed. E.g. let us create a subscription map with
MQTT-style wildcards (+ for ? and # for \*) but with the dot as the subtopic
separator:

```rust
let mut smap: SubMap<Client> =
    SubMap::new().separator('.').match_any("+").wildcard("#");
```

Note that "/topic/x", "topic/x" and "topic//x" are 3 different topics. If
any kind of normalization is required, it should be done manually, before
calling SubMap functions.

## Broadcast map

```rust
let mut bmap: BroadcastMap<Client> = BroadcastMap::new();
```

Does the opposite job - clients are registered with regular names, while
"get\_clients\_by\_mask" function returns clients, which match the mask.

Note: the default separator is dot.

## ACL map

```rust
let mut acl_map = AclMap::new();
```

SubMap-based high-speed access control lists checker. Uses SubMap algorithm
with a single unit "client" to verify various access control lists.

### Usage


## Cargo crate

<https://crates.io/crates/submap>

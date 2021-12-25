# submap

B-Tree map for pub/sub services.

## Create a new subscription map

```rust
let mut smap: SubMap<Client> = SubMap::new();
```

where "Client" is a pub/sub client type, which is usually either a channel or a
structure which contains a channel or locked socket or anything else, required
to work with the client.

The client type MUST provide traits Hash, Eq and Clone.

## Separators and wildcards

SubMap supports the following masks:

this/is/a/topic - single topic subscription
this/?/a/topic - all topics which match the pattern and have any 2nd chunk
this/is/\** - all subtopics of "this/is"
\* - all topics

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

## Usage

All clients must be registered in the map, before they can
subscribe/unsubscribe. Use "register_client" function for this.

When "unregister_client" is called, it also automatically unsubscribes the
client from all the subscribed topics.

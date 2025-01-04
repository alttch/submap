<h2>
  submap
  <a href="https://crates.io/crates/submap"><img alt="crates.io page" src="https://img.shields.io/crates/v/submap.svg"></img></a>
  <a href="https://docs.rs/submap"><img alt="docs.rs page" src="https://docs.rs/submap/badge.svg"></img></a>
  <a href="https://github.com/alttch/submap/actions/workflows/ci.yml">
    <img alt="GitHub Actions CI" src="https://github.com/alttch/submap/actions/workflows/ci.yml/badge.svg"></img>
  </a>
</h2>

B-tree map for pub/sub services.

## Subscription map

### Usage

```rust
use submap::SubMap;

type Client = String;

let mut smap: SubMap<Client> = SubMap::new();
```

where "Client" is a pub/sub client type, which is usually either a channel or a
structure which contains a channel or locked socket or anything else, required
to work with the client.

The client type MUST provide traits Ord, Eq and Clone.

All clients must be registered in the map, before they can
subscribe/unsubscribe. Use "register\_client" function for this.

When "unregister\_client" is called, it also automatically unsubscribes the
client from all the subscribed topics.

### Separators and wildcards

[`SubMap`] supports the following masks:

* this/is/a/topic - single topic subscription
* this/?/a/topic - all topics which match the pattern (2nd chunk - any value)
* this/is/\* - all subtopics of "this/is"
* \* - all topics

Service symbols can be changed. E.g. let us create a subscription map with
MQTT-style wildcards (+ for ? and # for \*) but with the dot as the subtopic
separator:

```rust
use submap::SubMap;

type Client = String;

let mut smap: SubMap<Client> =
    SubMap::new().separator('.').match_any("+").wildcard("#");
```

Note that "/topic/x", "topic/x" and "topic//x" are 3 different topics. If
any kind of normalization is required, it should be done manually, before
calling [`SubMap`] functions.

### Formulas

[`SubMap`] supports formulas, which are used both to subscribe to a topic by
formula or to get a list of clients which match one.

Formulas are non-standard pub/sub functionality and are useful when a client
want to subscribe to topics which have got e.g. some importance level. Instead
of subscribing to all level topics, a client can subscribe to one topic with a
formula:

```rust
use submap::SubMap;

type Client = String;

let mut smap: SubMap<Client> =
    SubMap::new().separator('/').match_any("+").wildcard("#").formula_prefix("!");
let client1 = "client1".to_owned();
smap.register_client(&client1);
smap.subscribe("some/!ge(2)/topic", &client1);
assert_eq!(smap.get_subscribers("some/1/topic").len(), 0);
assert_eq!(smap.get_subscribers("some/2/topic").len(), 1);
assert_eq!(smap.get_subscribers("some/3/topic").len(), 1);
```

See more: [`mkmf::Formula`].

### Regular expressions

[`SubMap`] supports regular expressions in subtopic names.

Regular expressions are non-standard pub/sub functionality, are pretty slow
(especially for subscribe/unsubscribe operations) and should be used with
caution. A regular expression can not contain the separator symbol.

```rust
use submap::SubMap;

type Client = String;

let mut smap: SubMap<Client> =
    SubMap::new().separator('/').match_any("+").wildcard("#").regex_prefix("~");
let client1 = "client1".to_owned();
smap.register_client(&client1);
smap.subscribe("some/~subtopic[0-9]+/topic", &client1);
assert_eq!(smap.get_subscribers("some/subtopic1/topic").len(), 1);
assert_eq!(smap.get_subscribers("some/subtopic2/topic").len(), 1);
assert_eq!(smap.get_subscribers("some/subtopic333/topic").len(), 1);
assert_eq!(smap.get_subscribers("some/subtopicx/topic").len(), 0);
```

## Broadcast map

```rust
use submap::BroadcastMap;

type Client = String;

let mut bmap: BroadcastMap<Client> = BroadcastMap::new();
```

Does the opposite job - clients are registered with regular names, while
"get\_clients\_by\_mask" function returns clients, which match the mask.

Note: the default separator is dot.

## ACL map

```rust
let mut acl_map = submap::AclMap::new();
```

SubMap-based high-speed access control lists checker. Uses SubMap algorithm
with a single unit "client" to verify various access control lists.

### Crate features

* **indexmap** switches the engine to
[indexmap](https://crates.io/crates/indexmap) (the default is based on
*std::collections::BTreeMap/BTreeSet*), requires Hash trait implemented for map
clients.

The current engine can be obtained from

```rust
use submap::types::ENGINE;

dbg!(ENGINE); // std-btree or indexmap
```

## MSRV

1.76.0

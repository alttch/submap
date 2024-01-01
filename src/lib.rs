#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "README.md" ) ) ]
mod submap;
pub use crate::submap::SubMap;

mod broadcastmap;
pub use crate::broadcastmap::BroadcastMap;

mod aclmap;
pub use crate::aclmap::AclMap;

#[cfg(feature = "digest")]
pub mod digest;

#[cfg(feature = "native-digest")]
#[path = "native_digest.rs"]
pub mod digest;

#[cfg(not(feature = "indexmap"))]
pub mod types {
    use std::collections::{BTreeMap, BTreeSet};

    pub type Set<V> = BTreeSet<V>;
    pub type Map<K, V> = BTreeMap<K, V>;
    pub trait Client: Ord + Eq + Clone {}
    impl<T: Ord + Eq + Clone> Client for T {}

    pub const ENGINE: &str = "std-btree";
}

#[cfg(feature = "indexmap")]
pub mod types {
    use indexmap::{IndexMap, IndexSet};
    use std::hash::Hash;

    pub type Set<V> = IndexSet<V>;
    pub type Map<K, V> = IndexMap<K, V>;
    pub trait Client: Ord + Eq + Clone + Hash {}
    impl<T: Ord + Eq + Clone + Hash> Client for T {}

    pub const ENGINE: &str = "indexmap";
}

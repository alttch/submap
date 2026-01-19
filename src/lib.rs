#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "README.md" ) ) ]
mod submap;
pub use crate::submap::SubMap;

mod broadcastmap;
pub use crate::broadcastmap::BroadcastMap;

mod aclmap;
pub use crate::aclmap::AclMap;

#[cfg(feature = "native-digest")]
#[path = "native_digest.rs"]
pub mod digest;

#[cfg(all(feature = "digest", not(feature = "native-digest")))]
pub mod digest;

pub mod mkmf;

#[cfg(not(feature = "indexmap"))]
pub mod types {
    use std::collections::{BTreeMap, BTreeSet};

    pub type Set<V> = BTreeSet<V>;
    pub type Map<K, V> = BTreeMap<K, V>;
    pub trait Client: Ord + Clone {}
    impl<T: Ord + Clone> Client for T {}

    pub const ENGINE: &str = "std-btree";
}

#[cfg(feature = "indexmap")]
pub mod types {
    use indexmap::{IndexMap, IndexSet};
    use std::hash::Hash;

    pub type Set<V> = IndexSet<V>;
    pub type Map<K, V> = IndexMap<K, V>;
    pub trait Client: Ord + Clone + Hash {}
    impl<T: Ord + Clone + Hash> Client for T {}

    pub const ENGINE: &str = "indexmap";
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("formula parse: {0}")]
    FormulaParseError(String),
}

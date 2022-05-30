use crate::submap::SubMap;

#[derive(Debug, Default, Clone)]
pub struct AclMap {
    smap: SubMap<()>,
}

impl AclMap {
    #[inline]
    pub fn new() -> Self {
        let mut acl_map = Self::default();
        acl_map.smap.register_client(&());
        acl_map
    }
    #[inline]
    pub fn separator(mut self, separator: char) -> Self {
        self.smap = self.smap.separator(separator);
        self
    }
    #[inline]
    pub fn wildcard(mut self, wildcard: &str) -> Self {
        self.smap = self.smap.wildcard(wildcard);
        self
    }
    #[inline]
    pub fn match_any(mut self, match_any: &str) -> Self {
        self.smap = self.smap.match_any(match_any);
        self
    }
    #[inline]
    pub fn set(&mut self, topic: &str) {
        self.smap.subscribe(topic, &());
    }
    #[inline]
    pub fn matches(&self, topic: &str) -> bool {
        self.smap.is_subscribed(topic)
    }
}

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::str::Split;

#[derive(Debug, Clone)]
struct Broadcast<C> {
    childs: HashMap<String, Broadcast<C>>,
    childs_any: Option<Box<Broadcast<C>>>,
    members: HashSet<C>,
    members_wildcard: HashSet<C>,
}

impl<C> Broadcast<C> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.childs.is_empty() && self.members.is_empty()
    }
}

impl<C> Default for Broadcast<C> {
    fn default() -> Self {
        Self {
            childs: <_>::default(),
            childs_any: <_>::default(),
            members: <_>::default(),
            members_wildcard: <_>::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastMap<C> {
    broadcasts: Broadcast<C>,
    separator: char,
    match_any: String,
    wildcard: String,
}

impl<C> Default for BroadcastMap<C> {
    fn default() -> Self {
        Self {
            broadcasts: Broadcast::default(),
            separator: '.',
            match_any: "?".to_owned(),
            wildcard: "*".to_owned(),
        }
    }
}

impl<C> BroadcastMap<C>
where
    C: Hash + Eq + Clone,
{
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
    #[inline]
    pub fn separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }
    #[inline]
    pub fn match_any(mut self, match_any: &str) -> Self {
        self.match_any = match_any.to_owned();
        self
    }
    #[inline]
    pub fn wildcard(mut self, wildcard: &str) -> Self {
        self.wildcard = wildcard.to_owned();
        self
    }
    #[inline]
    pub fn register_client(&mut self, name: &str, client: &C) {
        register_broadcast_client_rec(&mut self.broadcasts, name.split(self.separator), client);
    }
    #[inline]
    pub fn unregister_client(&mut self, name: &str, client: &C) {
        unregister_broadcast_client_rec(&mut self.broadcasts, name.split(self.separator), client);
    }
    pub fn get_clients_by_mask(&self, mask: &str) -> HashSet<C> {
        let mut result = HashSet::new();
        get_broadcast_clients_rec(
            &self.broadcasts,
            mask.split(self.separator),
            &mut result,
            &self.wildcard,
            &self.match_any,
        );
        result
    }
}

fn get_broadcast_clients_rec<C>(
    broadcast: &Broadcast<C>,
    mut sp: Split<char>,
    result: &mut HashSet<C>,
    wildcard: &str,
    match_any: &str,
) where
    C: Hash + Eq + Clone,
{
    if let Some(chunk) = sp.next() {
        if chunk == wildcard {
            result.extend(broadcast.members_wildcard.clone());
        } else if chunk == match_any {
            if let Some(ref child) = broadcast.childs_any {
                get_broadcast_clients_rec(child, sp, result, wildcard, match_any);
            }
        } else if let Some(child) = broadcast.childs.get(chunk) {
            get_broadcast_clients_rec(child, sp, result, wildcard, match_any);
        }
    } else {
        result.extend(broadcast.members.clone());
    }
}

fn register_broadcast_client_rec<C>(broadcast: &mut Broadcast<C>, mut sp: Split<char>, client: &C)
where
    C: Hash + Eq + Clone,
{
    if let Some(chunk) = sp.next() {
        broadcast.members_wildcard.insert(client.clone());
        if let Some(c) = broadcast.childs.get_mut(chunk) {
            register_broadcast_client_rec(c, sp.clone(), client);
        } else {
            let mut child = Broadcast::default();
            register_broadcast_client_rec(&mut child, sp.clone(), client);
            broadcast.childs.insert(chunk.to_owned(), child);
        }
        if let Some(ref mut c) = broadcast.childs_any {
            register_broadcast_client_rec(c, sp, client);
        } else {
            let mut child = Broadcast::default();
            register_broadcast_client_rec(&mut child, sp, client);
            broadcast.childs_any.replace(Box::new(child));
        }
    } else {
        broadcast.members.insert(client.clone());
    }
}

fn unregister_broadcast_client_rec<C>(broadcast: &mut Broadcast<C>, mut sp: Split<char>, client: &C)
where
    C: Hash + Eq + Clone,
{
    if let Some(chunk) = sp.next() {
        broadcast.members_wildcard.remove(client);
        if let Some(c) = broadcast.childs.get_mut(chunk) {
            unregister_broadcast_client_rec(c, sp.clone(), client);
            if c.is_empty() {
                broadcast.childs.remove(chunk);
            }
        }
        if let Some(ref mut c) = broadcast.childs_any {
            unregister_broadcast_client_rec(c, sp, client);
            if c.is_empty() {
                broadcast.childs_any = None;
            }
        }
    } else {
        broadcast.members.remove(client);
    }
}

#[cfg(test)]
mod test {
    use super::BroadcastMap;
    #[test]
    fn test_broadcast() {
        let mut bmap: BroadcastMap<u32> = BroadcastMap::new().separator('/');
        let client1: u32 = 1;
        let client2: u32 = 2;
        let client3: u32 = 3;
        let client4: u32 = 4;
        let client5: u32 = 5;
        bmap.register_client("this/is/a", &client1);
        bmap.register_client("this/is/b", &client2);
        bmap.register_client("this/is", &client3);
        bmap.register_client("this", &client4);
        bmap.register_client("that/is/a", &client5);
        let clients = bmap.get_clients_by_mask("this/is/*");
        assert!(clients.contains(&client1));
        assert!(clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/is");
        assert!(!clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this");
        assert!(!clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/*");
        assert!(clients.contains(&client1));
        assert!(clients.contains(&client2));
        assert!(clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/is/*");
        assert!(clients.contains(&client1));
        assert!(clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("*");
        assert!(clients.contains(&client1));
        assert!(clients.contains(&client2));
        assert!(clients.contains(&client3));
        assert!(clients.contains(&client4));
        assert!(clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/is/a/*");
        assert!(!clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/is/a/?");
        assert!(!clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/?/a");
        assert!(clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("this/?/?");
        assert!(clients.contains(&client1));
        assert!(clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(!clients.contains(&client5));
        let clients = bmap.get_clients_by_mask("?/is/a");
        assert!(clients.contains(&client1));
        assert!(!clients.contains(&client2));
        assert!(!clients.contains(&client3));
        assert!(!clients.contains(&client4));
        assert!(clients.contains(&client5));
        bmap.unregister_client("this/is/a", &client1);
        bmap.unregister_client("this/is/b", &client2);
        bmap.unregister_client("this/is", &client3);
        bmap.unregister_client("this", &client4);
        bmap.unregister_client("that/is/a", &client5);
        assert!(bmap.broadcasts.is_empty());
    }
}

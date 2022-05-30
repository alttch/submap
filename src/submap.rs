use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::str::Split;

#[derive(Debug, Clone)]
struct Subscription<C> {
    subscribers: HashSet<C>,
    subtopics: HashMap<String, Subscription<C>>,
    subtopics_any: Option<Box<Subscription<C>>>, // ?
    sub_any: HashSet<C>,                         // *
}

impl<C> Default for Subscription<C> {
    fn default() -> Self {
        Self {
            subscribers: <_>::default(),
            subtopics: <_>::default(),
            subtopics_any: None,
            sub_any: <_>::default(),
        }
    }
}

impl<C> Subscription<C> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.subscribers.is_empty()
            && self.subtopics.is_empty()
            && self.subtopics_any.is_none()
            && self.sub_any.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SubMap<C> {
    subscriptions: Subscription<C>,
    subscribed_topics: HashMap<C, HashSet<String>>,
    subscription_count: usize,
    separator: char,
    match_any: HashSet<String>,
    wildcard: HashSet<String>,
}

impl<C> Default for SubMap<C> {
    fn default() -> Self {
        Self {
            subscriptions: <_>::default(),
            subscribed_topics: <_>::default(),
            subscription_count: 0,
            separator: '/',
            match_any: vec!["?".to_owned()].into_iter().collect(),
            wildcard: vec!["*".to_owned()].into_iter().collect(),
        }
    }
}

impl<C> SubMap<C>
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
    pub fn wildcard(mut self, wildcard: &str) -> Self {
        self.wildcard = vec![wildcard.to_owned()].into_iter().collect();
        self
    }
    #[inline]
    pub fn match_any(mut self, match_any: &str) -> Self {
        self.match_any = vec![match_any.to_owned()].into_iter().collect();
        self
    }
    #[inline]
    pub fn wildcard_multiple(mut self, wildcard_multiple: &[&str]) -> Self {
        self.wildcard = wildcard_multiple.iter().map(|&v| v.to_owned()).collect();
        self
    }
    #[inline]
    pub fn match_any_multiple(mut self, match_any_multiple: &[&str]) -> Self {
        self.match_any = match_any_multiple.iter().map(|&v| v.to_owned()).collect();
        self
    }
    #[inline]
    pub fn list_clients(&self) -> Vec<C> {
        self.subscribed_topics.keys().cloned().collect()
    }
    #[inline]
    pub fn list_topics(&self, client: &C) -> Vec<&str> {
        if let Some(topics) = self.subscribed_topics.get(client) {
            topics.iter().map(String::as_str).collect()
        } else {
            Vec::new()
        }
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.subscribed_topics.is_empty()
    }
    pub fn register_client(&mut self, client: &C) -> bool {
        if self.subscribed_topics.contains_key(client) {
            false
        } else {
            self.subscribed_topics
                .insert(client.clone(), HashSet::new());
            true
        }
    }
    pub fn unregister_client(&mut self, client: &C) -> bool {
        if let Some(client_topics) = self.subscribed_topics.remove(client) {
            for topic in client_topics {
                unsubscribe_rec(
                    &mut self.subscriptions,
                    topic.split(self.separator),
                    client,
                    &self.wildcard,
                    &self.match_any,
                );
                self.subscription_count -= 1;
            }
            true
        } else {
            false
        }
    }
    pub fn subscribe(&mut self, topic: &str, client: &C) -> bool {
        self.subscribed_topics
            .get_mut(client)
            .map_or(false, |client_topics| {
                if !client_topics.contains(topic) {
                    subscribe_rec(
                        &mut self.subscriptions,
                        topic.split(self.separator),
                        client,
                        &self.wildcard,
                        &self.match_any,
                    );
                    client_topics.insert(topic.to_owned());
                    self.subscription_count += 1;
                }
                true
            })
    }
    pub fn unsubscribe(&mut self, topic: &str, client: &C) -> bool {
        self.subscribed_topics
            .get_mut(client)
            .map_or(false, |client_topics| {
                if client_topics.contains(topic) {
                    unsubscribe_rec(
                        &mut self.subscriptions,
                        topic.split('/'),
                        client,
                        &self.wildcard,
                        &self.match_any,
                    );
                    client_topics.remove(topic);
                    self.subscription_count -= 1;
                }
                true
            })
    }
    #[inline]
    pub fn get_subscribers(&self, topic: &str) -> HashSet<C> {
        let mut result = HashSet::new();
        get_subscribers_rec(
            &self.subscriptions,
            topic.split(self.separator),
            &mut result,
        );
        result
    }
    #[inline]
    pub fn is_subscribed(&self, topic: &str) -> bool {
        is_subscribed_rec(&self.subscriptions, topic.split(self.separator))
    }
    #[inline]
    pub fn subscription_count(&self) -> usize {
        self.subscription_count
    }
    #[inline]
    pub fn client_count(&self) -> usize {
        self.subscribed_topics.len()
    }
}

fn subscribe_rec<C>(
    subscription: &mut Subscription<C>,
    mut sp: Split<char>,
    client: &C,
    wildcard: &HashSet<String>,
    match_any: &HashSet<String>,
) where
    C: Hash + Eq + Clone,
{
    if let Some(topic) = sp.next() {
        if wildcard.contains(topic) {
            subscription.sub_any.insert(client.clone());
        } else if match_any.contains(topic) {
            if let Some(ref mut sub) = subscription.subtopics_any {
                subscribe_rec(sub, sp, client, wildcard, match_any);
            } else {
                let mut sub = Subscription::default();
                subscribe_rec(&mut sub, sp, client, wildcard, match_any);
                subscription.subtopics_any = Some(Box::new(sub));
            }
        } else if let Some(sub) = subscription.subtopics.get_mut(topic) {
            subscribe_rec(sub, sp, client, wildcard, match_any);
        } else {
            let mut sub = Subscription::default();
            subscribe_rec(&mut sub, sp, client, wildcard, match_any);
            subscription.subtopics.insert(topic.to_owned(), sub);
        }
    } else {
        subscription.subscribers.insert(client.clone());
    }
}

fn unsubscribe_rec<C>(
    subscription: &mut Subscription<C>,
    mut sp: Split<char>,
    client: &C,
    wildcard: &HashSet<String>,
    match_any: &HashSet<String>,
) where
    C: Hash + Eq,
{
    if let Some(topic) = sp.next() {
        if wildcard.contains(topic) {
            subscription.sub_any.remove(client);
        } else if match_any.contains(topic) {
            if let Some(ref mut sub) = subscription.subtopics_any {
                unsubscribe_rec(sub, sp, client, wildcard, match_any);
                if sub.is_empty() {
                    subscription.subtopics_any = None;
                }
            }
        } else if let Some(sub) = subscription.subtopics.get_mut(topic) {
            unsubscribe_rec(sub, sp, client, wildcard, match_any);
            if sub.is_empty() {
                subscription.subtopics.remove(topic);
            }
        }
    } else {
        subscription.subscribers.remove(client);
    }
}

fn get_subscribers_rec<C>(
    subscription: &Subscription<C>,
    mut sp: Split<char>,
    result: &mut HashSet<C>,
) where
    C: Hash + Eq + Clone,
{
    if let Some(topic) = sp.next() {
        result.extend(subscription.sub_any.clone());
        if let Some(sub) = subscription.subtopics.get(topic) {
            get_subscribers_rec(sub, sp.clone(), result);
        }
        if let Some(ref sub) = subscription.subtopics_any {
            get_subscribers_rec(sub, sp, result);
        }
    } else {
        result.extend(subscription.subscribers.clone());
    }
}

fn is_subscribed_rec<C>(subscription: &Subscription<C>, mut sp: Split<char>) -> bool
where
    C: Hash + Eq + Clone,
{
    if let Some(topic) = sp.next() {
        if !subscription.sub_any.is_empty() {
            return true;
        }
        if let Some(sub) = subscription.subtopics.get(topic) {
            if is_subscribed_rec(sub, sp.clone()) {
                return true;
            }
        }
        if let Some(ref sub) = subscription.subtopics_any {
            if is_subscribed_rec(sub, sp) {
                return true;
            }
        }
    } else if !subscription.subscribers.is_empty() {
        return true;
    }
    false
}

#[cfg(test)]
mod test {
    use super::SubMap;
    #[test]
    fn test_sub() {
        let mut smap: SubMap<String> = SubMap::new().match_any("+").wildcard("#");
        let client1 = "test1".to_owned();
        assert!(smap.register_client(&client1));
        assert!(smap.subscribe("unit/tests/test1", &client1));
        assert!(smap.subscribe("unit/tests/test2", &client1));
        assert!(smap.subscribe("unit/tests/test3", &client1));
        assert!(smap.unregister_client(&client1));
        let client2 = "test2".to_owned();
        assert!(smap.register_client(&client2));
        assert!(smap.subscribe("unit/+/test2", &client2));
        assert!(smap.subscribe("unit/zzz/test2", &client2));
        assert!(smap.unsubscribe("unit/zzz/test2", &client2));
        let client3 = "test3".to_owned();
        assert!(smap.register_client(&client3));
        assert!(smap.subscribe("unit/+/+/+", &client3));
        assert!(smap.unsubscribe("unit/+/+/+", &client3));
        let client4 = "test4".to_owned();
        assert!(smap.register_client(&client4));
        assert!(smap.subscribe("unit/#", &client4));
        let subs = smap.get_subscribers("unit/tests/test2");
        assert_eq!(subs.len(), 2);
        assert_eq!(subs.contains(&client2), true);
        assert_eq!(subs.contains(&client4), true);
        let subs = smap.get_subscribers("unit/tests");
        assert_eq!(subs.len(), 1);
        assert!(smap.subscribe("#", &client4));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 1);
        assert_eq!(subs.contains(&client4), true);
        assert!(smap.unsubscribe("#", &client4));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 0);
        smap.unregister_client(&client1);
        smap.unregister_client(&client2);
        smap.unregister_client(&client3);
        smap.unregister_client(&client4);
        assert!(smap.register_client(&client1));
        assert!(smap.register_client(&client2));
        assert!(smap.subscribe("unit/tests/#", &client1));
        assert!(smap.subscribe("unit/+/#", &client2));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 0);
        let subs = smap.get_subscribers("unit/tests");
        assert_eq!(subs.len(), 0);
        let subs = smap.get_subscribers("unit/tests/xxx");
        assert_eq!(subs.contains(&client1), true);
        assert_eq!(subs.contains(&client2), true);
        assert_eq!(subs.len(), 2);
        let subs = smap.get_subscribers("unit/tests/xxx/yyy");
        assert_eq!(subs.len(), 2);
        assert_eq!(subs.contains(&client1), true);
        assert_eq!(subs.contains(&client2), true);
        assert!(smap.subscribe("unit/#", &client1));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 0);
        let subs = smap.get_subscribers("unit/tests");
        assert_eq!(subs.len(), 1);
        assert!(smap.subscribe("#", &client1));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 1);
        let subs = smap.get_subscribers("unit/tests");
        assert_eq!(subs.len(), 1);
        smap.unregister_client(&client1);
        smap.unregister_client(&client2);
        assert!(smap.subscriptions.is_empty());
    }
}

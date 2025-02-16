use std::str::Split;

use crate::mkmf::{Formula, MapKeysMatchFormula as _};
#[allow(clippy::wildcard_imports)]
use crate::types::*;

#[derive(Debug, Clone)]
struct RegexSubscription<C> {
    regex: regex::Regex,
    sub: Subscription<C>,
}

#[derive(Debug, Clone)]
struct Subscription<C> {
    subscribers: Set<C>,
    subtopics: Map<String, Subscription<C>>,
    subtopics_by_formula: Map<Formula, Subscription<C>>,
    subtopics_by_regex: Vec<RegexSubscription<C>>,
    subtopics_any: Option<Box<Subscription<C>>>, // ?
    sub_any: Set<C>,                             // *
}

impl<C> Default for Subscription<C> {
    fn default() -> Self {
        Self {
            subscribers: <_>::default(),
            subtopics: <_>::default(),
            subtopics_by_formula: <_>::default(),
            subtopics_by_regex: <_>::default(),
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
            && self.subtopics_by_formula.is_empty()
            && self.subtopics_by_regex.is_empty()
            && self.subtopics_any.is_none()
            && self.sub_any.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SubMap<C> {
    subscriptions: Subscription<C>,
    subscribed_topics: Map<C, Set<String>>,
    subscription_count: usize,
    separator: char,
    formula_prefix: Option<String>,
    regex_prefix: Option<String>,
    match_any: Set<String>,
    wildcard: Set<String>,
}

impl<C> Default for SubMap<C> {
    fn default() -> Self {
        Self {
            subscriptions: <_>::default(),
            subscribed_topics: <_>::default(),
            subscription_count: 0,
            separator: '/',
            formula_prefix: None,
            regex_prefix: None,
            match_any: vec!["?".to_owned()].into_iter().collect(),
            wildcard: vec!["*".to_owned()].into_iter().collect(),
        }
    }
}

impl<C> SubMap<C>
where
    C: Client,
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
    pub fn formula_prefix(mut self, prefix: &str) -> Self {
        self.formula_prefix = Some(prefix.to_owned());
        self
    }
    #[inline]
    pub fn regex_prefix(mut self, prefix: &str) -> Self {
        self.regex_prefix = Some(prefix.to_owned());
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
            self.subscribed_topics.insert(client.clone(), Set::new());
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
                    self.formula_prefix.as_deref(),
                    self.regex_prefix.as_deref(),
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
                        self.formula_prefix.as_deref(),
                        self.regex_prefix.as_deref(),
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
                        topic.split(self.separator),
                        client,
                        &self.wildcard,
                        &self.match_any,
                        self.formula_prefix.as_deref(),
                        self.regex_prefix.as_deref(),
                    );
                    client_topics.remove(topic);
                    self.subscription_count -= 1;
                }
                true
            })
    }
    pub fn unsubscribe_all(&mut self, client: &C) -> bool {
        if let Some(client_topics) = self.subscribed_topics.get_mut(client) {
            for topic in &*client_topics {
                unsubscribe_rec(
                    &mut self.subscriptions,
                    topic.split(self.separator),
                    client,
                    &self.wildcard,
                    &self.match_any,
                    self.formula_prefix.as_deref(),
                    self.regex_prefix.as_deref(),
                );
                self.subscription_count -= 1;
            }
            client_topics.clear();
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn get_subscribers(&self, topic: &str) -> Set<C> {
        let mut result = Set::new();
        get_subscribers_rec(
            &self.subscriptions,
            topic.split(self.separator),
            self.formula_prefix.as_deref(),
            self.regex_prefix.as_deref(),
            &mut result,
        );
        result
    }
    #[inline]
    pub fn is_subscribed(&self, topic: &str) -> bool {
        is_subscribed_rec(
            &self.subscriptions,
            self.formula_prefix.as_deref(),
            self.regex_prefix.as_deref(),
            topic.split(self.separator),
        )
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

#[allow(clippy::too_many_lines)]
fn subscribe_rec<C>(
    subscription: &mut Subscription<C>,
    mut sp: Split<char>,
    client: &C,
    wildcard: &Set<String>,
    match_any: &Set<String>,
    formula_prefix: Option<&str>,
    regex_prefix: Option<&str>,
) where
    C: Client,
{
    if let Some(topic) = sp.next() {
        if wildcard.contains(topic) {
            subscription.sub_any.insert(client.clone());
        } else if match_any.contains(topic) {
            if let Some(ref mut sub) = subscription.subtopics_any {
                subscribe_rec(
                    sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
            } else {
                let mut sub = Subscription::default();
                subscribe_rec(
                    &mut sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
                subscription.subtopics_any = Some(Box::new(sub));
            }
        } else if let Some(formula) = formula_prefix.and_then(|p| topic.strip_prefix(p)) {
            let Ok(formula_parsed) = formula.parse::<Formula>() else {
                return;
            };
            if let Some(sub) = subscription.subtopics_by_formula.get_mut(&formula_parsed) {
                subscribe_rec(
                    sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
            } else {
                let mut sub = Subscription::default();
                subscribe_rec(
                    &mut sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
                subscription
                    .subtopics_by_formula
                    .insert(formula_parsed, sub);
            }
        } else if let Some(regex) = regex_prefix.and_then(|p| topic.strip_prefix(p)) {
            if let Ok(regex) = regex::Regex::new(regex) {
                let pos = subscription
                    .subtopics_by_regex
                    .iter()
                    .position(|rs| rs.regex.as_str() == regex.as_str());
                if let Some(pos) = pos {
                    subscribe_rec(
                        &mut subscription.subtopics_by_regex[pos].sub,
                        sp,
                        client,
                        wildcard,
                        match_any,
                        formula_prefix,
                        regex_prefix,
                    );
                } else {
                    let mut sub = Subscription::default();
                    subscribe_rec(
                        &mut sub,
                        sp,
                        client,
                        wildcard,
                        match_any,
                        formula_prefix,
                        regex_prefix,
                    );
                    subscription
                        .subtopics_by_regex
                        .push(RegexSubscription { regex, sub });
                }
            }
        } else if let Some(sub) = subscription.subtopics.get_mut(topic) {
            subscribe_rec(
                sub,
                sp,
                client,
                wildcard,
                match_any,
                formula_prefix,
                regex_prefix,
            );
        } else {
            let mut sub = Subscription::default();
            subscribe_rec(
                &mut sub,
                sp,
                client,
                wildcard,
                match_any,
                formula_prefix,
                regex_prefix,
            );
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
    wildcard: &Set<String>,
    match_any: &Set<String>,
    formula_prefix: Option<&str>,
    regex_prefix: Option<&str>,
) where
    C: Client,
{
    if let Some(topic) = sp.next() {
        if wildcard.contains(topic) {
            subscription.sub_any.remove(client);
        } else if match_any.contains(topic) {
            if let Some(ref mut sub) = subscription.subtopics_any {
                unsubscribe_rec(
                    sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
                if sub.is_empty() {
                    subscription.subtopics_any = None;
                }
            }
        } else if let Some(formula) = formula_prefix.and_then(|p| topic.strip_prefix(p)) {
            let Ok(formula_parsed) = formula.parse::<Formula>() else {
                return;
            };
            if let Some(sub) = subscription.subtopics_by_formula.get_mut(&formula_parsed) {
                unsubscribe_rec(
                    sub,
                    sp,
                    client,
                    wildcard,
                    match_any,
                    formula_prefix,
                    regex_prefix,
                );
                if sub.is_empty() {
                    subscription.subtopics_by_formula.remove(&formula_parsed);
                }
            }
        } else if let Some(regex) = regex_prefix.and_then(|p| topic.strip_prefix(p)) {
            if let Ok(regex) = regex::Regex::new(regex) {
                let pos = subscription
                    .subtopics_by_regex
                    .iter()
                    .position(|rs| rs.regex.as_str() == regex.as_str());
                if let Some(pos) = pos {
                    let sub = &mut subscription.subtopics_by_regex[pos].sub;
                    unsubscribe_rec(
                        sub,
                        sp,
                        client,
                        wildcard,
                        match_any,
                        formula_prefix,
                        regex_prefix,
                    );
                    if sub.is_empty() {
                        subscription.subtopics_by_regex.remove(pos);
                    }
                }
            }
        } else if let Some(sub) = subscription.subtopics.get_mut(topic) {
            unsubscribe_rec(
                sub,
                sp,
                client,
                wildcard,
                match_any,
                formula_prefix,
                regex_prefix,
            );
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
    formula_prefix: Option<&str>,
    regex_prefix: Option<&str>,
    result: &mut Set<C>,
) where
    C: Client,
{
    if let Some(topic) = sp.next() {
        result.extend(subscription.sub_any.clone());
        if let Some(formula) = formula_prefix.and_then(|p| topic.strip_prefix(p)) {
            for sub in subscription.subtopics.values_match_key_formula(formula) {
                get_subscribers_rec(sub, sp.clone(), formula_prefix, regex_prefix, result);
            }
        } else if let Some(regex) = regex_prefix.and_then(|p| topic.strip_prefix(p)) {
            if let Ok(regex) = regex::Regex::new(regex) {
                for (name, sub) in &subscription.subtopics {
                    if regex.is_match(name) {
                        get_subscribers_rec(sub, sp.clone(), formula_prefix, regex_prefix, result);
                    }
                }
            }
        } else if let Some(sub) = subscription.subtopics.get(topic) {
            get_subscribers_rec(sub, sp.clone(), formula_prefix, regex_prefix, result);
        }
        if !subscription.subtopics_by_formula.is_empty() {
            for (formula, sub) in &subscription.subtopics_by_formula {
                if formula.matches(topic) {
                    get_subscribers_rec(sub, sp.clone(), formula_prefix, regex_prefix, result);
                }
            }
        }
        if !subscription.subtopics_by_regex.is_empty() {
            for rs in &subscription.subtopics_by_regex {
                if rs.regex.is_match(topic) {
                    get_subscribers_rec(&rs.sub, sp.clone(), formula_prefix, regex_prefix, result);
                }
            }
        }
        if let Some(ref sub) = subscription.subtopics_any {
            get_subscribers_rec(sub, sp, formula_prefix, regex_prefix, result);
        }
    } else {
        result.extend(subscription.subscribers.clone());
    }
}

fn is_subscribed_rec<C>(
    subscription: &Subscription<C>,
    formula_prefix: Option<&str>,
    regex_prefix: Option<&str>,
    mut sp: Split<char>,
) -> bool
where
    C: Ord + Eq + Clone,
{
    if let Some(topic) = sp.next() {
        if !subscription.sub_any.is_empty() {
            return true;
        }
        if let Some(formula) = formula_prefix.and_then(|p| topic.strip_prefix(p)) {
            for sub in subscription.subtopics.values_match_key_formula(formula) {
                if is_subscribed_rec(sub, formula_prefix, regex_prefix, sp.clone()) {
                    return true;
                }
            }
        } else if let Some(regex) = regex_prefix.and_then(|p| topic.strip_prefix(p)) {
            if let Ok(regex) = regex::Regex::new(regex) {
                for (name, sub) in &subscription.subtopics {
                    if regex.is_match(name)
                        && is_subscribed_rec(sub, formula_prefix, regex_prefix, sp.clone())
                    {
                        return true;
                    }
                }
            }
        } else if let Some(sub) = subscription.subtopics.get(topic) {
            if is_subscribed_rec(sub, formula_prefix, regex_prefix, sp.clone()) {
                return true;
            }
        }
        if !subscription.subtopics_by_formula.is_empty() {
            for (formula, sub) in &subscription.subtopics_by_formula {
                if formula.matches(topic)
                    && is_subscribed_rec(sub, formula_prefix, regex_prefix, sp.clone())
                {
                    return true;
                }
            }
        }
        if !subscription.subtopics_by_regex.is_empty() {
            for rs in &subscription.subtopics_by_regex {
                if rs.regex.is_match(topic)
                    && is_subscribed_rec(&rs.sub, formula_prefix, regex_prefix, sp.clone())
                {
                    return true;
                }
            }
        }
        if let Some(ref sub) = subscription.subtopics_any {
            if is_subscribed_rec(sub, formula_prefix, regex_prefix, sp) {
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
        assert!(subs.contains(&client2));
        assert!(subs.contains(&client4));
        let subs = smap.get_subscribers("unit/tests");
        assert_eq!(subs.len(), 1);
        assert!(smap.subscribe("#", &client4));
        let subs = smap.get_subscribers("unit");
        assert_eq!(subs.len(), 1);
        assert!(subs.contains(&client4));
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
        assert!(subs.contains(&client1));
        assert!(subs.contains(&client2));
        assert_eq!(subs.len(), 2);
        let subs = smap.get_subscribers("unit/tests/xxx/yyy");
        assert_eq!(subs.len(), 2);
        assert!(subs.contains(&client1));
        assert!(subs.contains(&client2));
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
    #[test]
    fn test_match_any() {
        let mut smap: SubMap<String> = SubMap::new().match_any("+").wildcard("#");
        let client1 = "client1".to_owned();
        smap.register_client(&client1);
        assert_eq!(smap.get_subscribers("abc/xxx").len(), 0);
        smap.subscribe("+/xxx", &client1);
        assert_eq!(smap.get_subscribers("abc/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("unix/zzz/xxx/222").len(), 0);
        smap.subscribe("+/zzz/+/222", &client1);
        assert_eq!(smap.get_subscribers("unix/zzz/xxx/222").len(), 1);
    }
    #[test]
    fn test_match_formula() {
        let mut smap: SubMap<String> = SubMap::new()
            .match_any("+")
            .wildcard("#")
            .formula_prefix("!");
        let client1 = "client1".to_owned();
        smap.register_client(&client1);
        assert_eq!(smap.get_subscribers("1/xxx").len(), 0);
        smap.subscribe("!ge(2)/xxx", &client1);
        assert_eq!(smap.get_subscribers("1/xxx").len(), 0);
        assert_eq!(smap.get_subscribers("2/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("3/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("unix/zzz/95/222").len(), 0);
        assert_eq!(smap.get_subscribers("unix/zzz/96/222").len(), 0);
        assert_eq!(smap.get_subscribers("unix/zzz/97/222").len(), 0);
        smap.subscribe("+/zzz/!ge(96)/222", &client1);
        assert_eq!(smap.get_subscribers("unix/zzz/95/222").len(), 0);
        assert_eq!(smap.get_subscribers("unix/zzz/96/222").len(), 1);
        assert_eq!(smap.get_subscribers("unix/zzz/97/222").len(), 1);
    }
    #[test]
    fn test_match_regex() {
        let mut smap: SubMap<String> = SubMap::new().match_any("+").wildcard("#").regex_prefix("~");
        let client1 = "client1".to_owned();
        smap.register_client(&client1);
        assert_eq!(smap.get_subscribers("test1/xxx").len(), 0);
        smap.subscribe("~^test\\d+$/xxx", &client1);
        assert_eq!(smap.get_subscribers("test1/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("test2/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("test3333/xxx").len(), 1);
        assert_eq!(smap.get_subscribers("test3333a/xxx").len(), 0);
        let client2 = "client2".to_owned();
        smap.register_client(&client2);
        smap.subscribe("~^test\\d+$/xxx", &client2);
        assert_eq!(smap.get_subscribers("test1/xxx").len(), 2);
        assert_eq!(smap.get_subscribers("test2/xxx").len(), 2);
        assert_eq!(smap.get_subscribers("test3333/xxx").len(), 2);
        assert_eq!(smap.get_subscribers("test3333a/xxx").len(), 0);
        smap.unsubscribe("~^test\\d+$/xxx", &client1);
        assert_eq!(smap.get_subscribers("test1/xxx").len(), 1);
        smap.unsubscribe("~^test\\d+$/xxx", &client2);
        assert_eq!(smap.get_subscribers("test1/xxx").len(), 0);
    }
}

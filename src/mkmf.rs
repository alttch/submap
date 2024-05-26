//! A module for matching keys/value keys in a map with a formula.
//!
//! Map keys must be strings
//!
//! Functions supported in formulas:
//!
//! - `eq(n)`: equal to n
//! - `ne(n)`: not equal to n
//! - `gt(n)`: greater than n
//! - `lt(n)`: less than n
//! - `ge(n)`: greater than or equal to n
//! - `le(n)`: less than or equal to n
//! - `ri(n..m)`: range from n to m
//!
//! A key is parsed as i64 before comparison.
//!
//! # Example
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use submap::mkmf::MapKeysMatchFormula as _;
//!
//! let mut h: BTreeMap<String, ()> = BTreeMap::new();
//! h.insert("hello".to_string(), ());
//! h.insert("world".to_string(), ());
//! h.insert("1".to_string(), ());
//! h.insert("2".to_string(), ());
//! h.insert("3".to_string(), ());
//! h.insert("4".to_string(), ());
//! h.insert("5".to_string(), ());
//! assert_eq!(
//!    h.keys_match_formula("ge(4)").collect::<Vec<&String>>(),
//!    ["4", "5"]);
//! ```
//!
//! If a key has a prefix, it can be parsed as a `prefix#function(value)`:
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use submap::mkmf::MapKeysMatchFormula as _;
//!
//! let mut h: BTreeMap<String, ()> = BTreeMap::new();
//! h.insert("hello".to_string(), ());
//! h.insert("world".to_string(), ());
//! h.insert("a1".to_string(), ());
//! h.insert("a2".to_string(), ());
//! h.insert("a3".to_string(), ());
//! h.insert("a4".to_string(), ());
//! h.insert("a5".to_string(), ());
//! assert_eq!(
//!   h.keys_match_formula("a#ge(4)").collect::<Vec<&String>>(),
//!   ["a4", "a5"]);
//! ```
use crate::Error;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

pub trait MapKeysMatchFormula<K, V> {
    fn keys_match_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a K>
    where
        K: 'a;
    fn values_match_key_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a V>
    where
        V: 'a;
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct Formula {
    prefix: Option<String>,
    calc: FormulaCalc,
}

impl Formula {
    pub fn matches<S>(&self, value: S) -> bool
    where
        S: AsRef<str>,
    {
        if let Some(ref prefix) = self.prefix {
            let Some(v) = value.as_ref().strip_prefix(prefix) else {
                return false;
            };
            return self.calc.matches(v);
        }
        self.calc.matches(value)
    }
}

impl FromStr for Formula {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '#');
        let mut prefix = parts.next();
        let formula = if let Some(f) = parts.next() {
            f
        } else {
            let p = prefix;
            prefix = None;
            p.ok_or_else(|| Error::FormulaParseError(format!("function not defined in {}", s)))?
        };
        let calc = formula.parse()?;
        Ok(Formula {
            prefix: prefix.map(ToOwned::to_owned),
            calc,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
enum FormulaCalc {
    Eq(i64),
    Ne(i64),
    Gt(i64),
    Lt(i64),
    Ge(i64),
    Le(i64),
    Ri(i64, i64),
}

impl FormulaCalc {
    fn matches<S>(&self, value: S) -> bool
    where
        S: AsRef<str>,
    {
        let Ok(value) = value.as_ref().parse::<i64>() else {
            return matches!(self, FormulaCalc::Ne(_));
        };
        match self {
            FormulaCalc::Eq(f) => value == *f,
            FormulaCalc::Ne(f) => value != *f,
            FormulaCalc::Gt(f) => value > *f,
            FormulaCalc::Lt(f) => value < *f,
            FormulaCalc::Ge(f) => value >= *f,
            FormulaCalc::Le(f) => value <= *f,
            FormulaCalc::Ri(f1, f2) => value >= *f1 && value <= *f2,
        }
    }
}

impl FromStr for FormulaCalc {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('(');
        let kind = parts
            .next()
            .ok_or_else(|| Error::FormulaParseError(format!("function not defined in {}", s)))?;
        let value = parts
            .next()
            .ok_or_else(|| Error::FormulaParseError(format!("value not defined in {}", s)))?;
        let Some(value) = value.strip_suffix(')') else {
            return Err(Error::FormulaParseError(format!(
                "bracket not closed in {}",
                s
            )));
        };
        macro_rules! parse_val {
            ($value:expr) => {
                $value.parse().map_err(|e| {
                    Error::FormulaParseError(format!("formula value parse error in {}: {}", s, e))
                })?
            };
        }
        match kind {
            "eq" => Ok(FormulaCalc::Eq(parse_val!(value))),
            "ne" => Ok(FormulaCalc::Ne(parse_val!(value))),
            "gt" => Ok(FormulaCalc::Gt(parse_val!(value))),
            "lt" => Ok(FormulaCalc::Lt(parse_val!(value))),
            "ge" => Ok(FormulaCalc::Ge(parse_val!(value))),
            "le" => Ok(FormulaCalc::Le(parse_val!(value))),
            "ri" => {
                let mut parts = value.split("..");
                let f1 = parse_val!(parts.next().ok_or_else(|| {
                    Error::FormulaParseError(format!("range first value not defined in {}", s))
                })?);
                let f2 = parse_val!(parts.next().ok_or_else(|| {
                    Error::FormulaParseError(format!("range second value not defined in {}", s))
                })?);
                Ok(FormulaCalc::Ri(f1, f2))
            }
            v => Err(Error::FormulaParseError(format!(
                "unknown function in {}: {}",
                s, v
            ))),
        }
    }
}

impl<K: std::hash::Hash + Eq, V, S: ::std::hash::BuildHasher> MapKeysMatchFormula<K, V>
    for HashMap<K, V, S>
where
    K: AsRef<str>,
{
    fn keys_match_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a K>
    where
        K: 'a,
    {
        let keys = self.keys();
        keys_match_formula(keys, formula)
    }
    fn values_match_key_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        values_match_key_formula(self.iter(), formula)
    }
}

impl<K, V> MapKeysMatchFormula<K, V> for BTreeMap<K, V>
where
    K: AsRef<str>,
{
    fn keys_match_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a K>
    where
        K: 'a,
    {
        let keys = self.keys();
        keys_match_formula(keys, formula)
    }
    fn values_match_key_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        values_match_key_formula(self.iter(), formula)
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S: ::std::hash::BuildHasher> MapKeysMatchFormula<K, V> for indexmap::IndexMap<K, V, S>
where
    K: AsRef<str>,
{
    fn keys_match_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a K>
    where
        K: 'a,
    {
        let keys = self.keys();
        keys_match_formula(keys, formula)
    }
    fn values_match_key_formula<'a>(&'a self, formula: &'a str) -> impl Iterator<Item = &'a V>
    where
        V: 'a,
    {
        values_match_key_formula(self.iter(), formula)
    }
}

fn keys_match_formula<'a, K, I>(keys: I, formula: &str) -> impl Iterator<Item = &'a K>
where
    K: AsRef<str> + 'a,
    I: Iterator<Item = &'a K>,
{
    let formula_parsed = formula.parse::<Formula>().ok();
    keys.filter(move |key| formula_parsed.as_ref().map_or(false, |f| f.matches(key)))
}

fn values_match_key_formula<'a, K, V, I>(iter: I, formula: &str) -> impl Iterator<Item = &'a V>
where
    K: AsRef<str> + 'a,
    I: Iterator<Item = (&'a K, &'a V)>,
    V: 'a,
{
    let formula_parsed = formula.parse::<Formula>().ok();
    iter.filter(move |(key, _)| formula_parsed.as_ref().map_or(false, |f| f.matches(key)))
        .map(|(_, value)| value)
}

#[allow(clippy::zero_sized_map_values)]
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::MapKeysMatchFormula as _;

    #[test]
    fn test_keys_matches_formula_eq() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("eq(1)").collect::<Vec<&String>>(),
            ["1"]
        );
    }
    #[test]
    fn test_keys_matches_formula_ne() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("ne(1)").collect::<Vec<&String>>(),
            ["2", "3", "4", "5", "hello", "world"]
        );
    }
    #[test]
    fn test_keys_matches_formula_gt() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("gt(3)").collect::<Vec<&String>>(),
            ["4", "5"]
        );
    }
    #[test]
    fn test_keys_matches_formula_gt_prefix() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("a1".to_string(), ());
        h.insert("a2".to_string(), ());
        h.insert("a3".to_string(), ());
        h.insert("a4".to_string(), ());
        h.insert("a5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("a#gt(3)").collect::<Vec<&String>>(),
            ["a4", "a5"]
        );
    }
    #[test]
    fn test_keys_matches_formula_lt() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("lt(3)").collect::<Vec<&String>>(),
            ["1", "2"]
        );
    }
    #[test]
    fn test_keys_matches_formula_ge() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("ge(3)").collect::<Vec<&String>>(),
            ["3", "4", "5"]
        );
    }
    #[test]
    fn test_keys_matches_formula_le() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("le(3)").collect::<Vec<&String>>(),
            ["1", "2", "3"]
        );
    }
    #[test]
    fn test_keys_matches_formula_ri() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("ri(2..4)").collect::<Vec<&String>>(),
            ["2", "3", "4"]
        );
    }
    #[test]
    fn test_keys_matches_formula_ri_prefix() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("a1".to_string(), ());
        h.insert("a2".to_string(), ());
        h.insert("a3".to_string(), ());
        h.insert("a4".to_string(), ());
        h.insert("a5".to_string(), ());
        assert_eq!(
            h.keys_match_formula("a#ri(2..4)").collect::<Vec<&String>>(),
            ["a2", "a3", "a4"]
        );
    }
    #[test]
    fn test_keys_matches_formula_invalid() {
        let mut h: BTreeMap<String, ()> = BTreeMap::new();
        h.insert("hello".to_string(), ());
        h.insert("world".to_string(), ());
        h.insert("1".to_string(), ());
        h.insert("2".to_string(), ());
        h.insert("3".to_string(), ());
        h.insert("4".to_string(), ());
        h.insert("5".to_string(), ());
        assert!(h
            .keys_match_formula("a#xxx(2..4)")
            .collect::<Vec<&String>>()
            .is_empty());
    }
}

use std::{collections::HashMap, iter::Iterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Tags(HashMap<String, i32>);

impl Tags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, name: impl AsRef<str>) -> bool {
        self.0.contains_key(name.as_ref())
    }

    pub fn value(&self, name: impl AsRef<str>) -> i32 {
        self.0.get(name.as_ref()).cloned().unwrap_or(0)
    }

    pub fn add(&mut self, name: impl Into<String>, value: i32) {
        *self.0.entry(name.into()).or_insert(0) += value;
    }

    pub fn merge_in(&mut self, other: &Self) {
        for (name, value) in &other.0 {
            self.add(name, *value);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    Has(String),
    Lt(Vec<ConditionOperand>),
    Lte(Vec<ConditionOperand>),
    Eq(Vec<ConditionOperand>),
    Ne(Vec<ConditionOperand>),
    Gte(Vec<ConditionOperand>),
    Gt(Vec<ConditionOperand>),
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionOperand {
    Constant(i32),
    Tag(String),
}

impl Condition {
    pub fn check(&self, tags: &Tags) -> bool {
        match self {
            Self::Has(tag) => tags.has(tag),
            Self::Lt(args) => perform_operation(tags, args, |a, b| a < b),
            Self::Lte(args) => perform_operation(tags, args, |a, b| a <= b),
            Self::Eq(args) => perform_operation(tags, args, |a, b| a == b),
            Self::Ne(args) => perform_operation(tags, args, |a, b| a != b),
            Self::Gte(args) => perform_operation(tags, args, |a, b| a >= b),
            Self::Gt(args) => perform_operation(tags, args, |a, b| a > b),
            Self::And(conds) => conds.iter().all(|c| c.check(tags)),
            Self::Or(conds) => conds.iter().any(|c| c.check(tags)),
            Self::Not(inner) => !inner.check(tags),
        }
    }
}

fn perform_operation(
    tags: &Tags,
    args: &[ConditionOperand],
    op: impl Fn(i32, i32) -> bool,
) -> bool {
    let resolved = args.iter().map(|arg| match arg {
        ConditionOperand::Constant(value) => *value,
        ConditionOperand::Tag(name) => tags.value(name),
    });

    let mut maybe_last: Option<i32> = None;
    for arg in resolved {
        if let Some(last) = maybe_last {
            if !op(last, arg) {
                return false;
            }
        }
        maybe_last = Some(arg);
    }
    true
}

#[cfg(test)]
mod test {
    #[test]
    fn perform_operation() {
        let mut tags = super::Tags(std::collections::HashMap::new());
        tags.add("foo", 1);
        tags.add("bar", 5);

        use super::ConditionOperand::Constant as Val;
        use super::ConditionOperand::Tag;
        assert!(super::perform_operation(
            &tags,
            &[Tag("foo".into()), Val(3), Tag("bar".into())],
            |a, b| a < b
        ));
        assert!(!super::perform_operation(
            &tags,
            &[Tag("foo".into()), Val(5), Tag("bar".into())],
            |a, b| a < b
        ));
    }
}

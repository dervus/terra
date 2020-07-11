use std::fmt;
use std::collections::HashSet;
use serde::de;

#[derive(Debug, Clone)]
pub enum Condition {
    Has(String),
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

pub fn check(cond: Option<&Condition>, tags: &HashSet<String>) -> bool {
    match cond {
        None => true,
        Some(Condition::Has(tag)) => tags.contains(tag),
        Some(Condition::And(inner)) => inner.iter().all(|c| check(Some(c), tags)),
        Some(Condition::Or(inner)) => inner.iter().any(|c| check(Some(c), tags)),
        Some(Condition::Not(inner)) => !check(Some(inner), tags),
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut seq = |op, iter: &[Condition]| {
            f.write_str("(")?;
            f.write_str(op)?;
            for c in iter {
                f.write_str(" ")?;
                c.fmt(f)?;
            }
            f.write_str(")")?;
            Ok(())
        };
        match self {
            Self::Has(tag) => f.write_str(tag),
            Self::And(inner) => seq("and", inner),
            Self::Or(inner) => seq("or", inner),
            Self::Not(inner) => {
                f.write_str("(not ")?;
                inner.fmt(f)?;
                f.write_str(")")?;
                Ok(())
            }
        }
    }
}

impl<'de> de::Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ConditionVisitor)
    }
}

struct ConditionVisitor;

impl<'de> de::Visitor<'de> for ConditionVisitor {
    type Value = Condition;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("terra tag condition expression")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Condition::Has(v))
    }

    fn visit_seq<A>(self, mut v: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>
    {
        let first: String = v.next_element()?.ok_or(de::Error::custom("every sequence must start with a string element"))?;
        let mut rest: Vec<Condition> = if let Some(hint) = v.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };
        while let Some(constraint) = v.next_element()? {
            rest.push(constraint)
        }
        match first.as_ref() {
            "and" => Ok(Condition::And(rest)),
            "or" => Ok(Condition::Or(rest)),
            "not" => {
                if rest.len() == 1 {
                    Ok(Condition::Not(Box::new(rest.into_iter().nth(0).unwrap())))
                } else {
                    Err(de::Error::custom("not condition must contain exactly one inner condition"))
                }
            }
            _ => Err(de::Error::custom(format!("invalid condition operator {:?}", first)))
        }
    }
}

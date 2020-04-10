use std::fmt;
use std::collections::HashSet;
use serde::Deserialize;
use serde::de;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Relations {
    pub requires: Option<Constraint>,
    pub provides: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Tag(String),
    All(Vec<Constraint>),
    Any(Vec<Constraint>),
}

impl Constraint {
    pub fn check(&self, tags: &HashSet<String>) -> bool {
        match self {
            Self::Tag(tag) => tags.contains(tag),
            Self::All(inner) => inner.iter().all(|c| c.check(tags)),
            Self::Any(inner) => inner.iter().any(|c| c.check(tags)),
        }
    }
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(tag) => {
                f.write_str(tag)?;
            }
            Self::All(inner) => {
                f.write_str("(all")?;
                for c in inner {
                    f.write_char(' ')?;
                    c.fmt(f)?;
                }
                f.write_char(')')?;
            }
            Self::Any(inner) => {
                f.write_str("(any")?;
                for c in inner {
                    f.write_char(' ')?;
                    c.fmt(f)?;
                }
                f.write_char(')')?;
            }
        }
        Ok(())
    }
}

impl<'de> de::Deserialize<'de> for Constraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ConstraintVisitor)
    }
}

struct ConstraintVisitor;

impl<'de> de::Visitor<'de> for ConstraintVisitor {
    type Value = Constraint;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("terra constraint expression")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_string(v.to_owned())
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Constraint::Tag(v))
    }

    fn visit_seq<A>(self, v: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>
    {
        let first: String = v.next_element()?.ok_or(de::Error::custom("every sequence must start with a string element"))?;
        let mut rest: Vec<Constraint> = if let Some(hint) = v.size_hint() {
            Vec::with_capacity(hint)
        } else {
            Vec::new()
        };
        while let Some(constraint) = v.next_element()? {
            rest.push(constraint)
        }
        match first.as_ref() {
            "all" => Ok(Constraint::All(rest)),
            "any" => Ok(Constraint::Any(rest)),
            _ => Err(de::Error::custom(format!("invalid constraint operator {:?}", first)))
        }
    }
}

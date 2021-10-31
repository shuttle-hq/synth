use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub type Object = Map<String, Value>;

use super::MergeStrategy;
use crate::schema::{BoolContent, Content, NumberContent, StringContent, ValueKindExt};

#[derive(Clone, Copy, Default)]
pub struct ValueMergeStrategy {
    pub depth: Option<i32>,
    pub replace: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Squashables {
    Content(Content),
    NumberContent(NumberContent),
    StringContent(StringContent),
    BoolContent(BoolContent),
}

impl Squashables {
    fn kind(&self) -> String {
        match self {
            Self::Content(Content::Number(number_content))
            | Self::NumberContent(number_content) => number_content.kind(),
            Self::Content(Content::String(string_content))
            | Self::StringContent(string_content) => string_content.kind(),
            Self::Content(Content::Bool(bool_content)) | Self::BoolContent(bool_content) => {
                bool_content.kind()
            }
            Self::Content(content) => content.kind(),
        }
    }

    fn into_object(self) -> Result<Object> {
        let content = match self {
            Self::Content(content) => content,
            Self::NumberContent(number_content) => Content::Number(number_content),
            Self::StringContent(string_content) => Content::String(string_content),
            Self::BoolContent(bool_content) => Content::Bool(bool_content),
        };
        let as_value = serde_json::to_value(&content)?;
        match as_value {
            Value::Object(object) => Ok(object),
            // SAFETY: assumes `Content::Null` is not in `Sqashables`
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for ValueMergeStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValueMergeStrategy(depth = {:?}, replace = {})",
            self.depth, self.replace
        )
    }
}

impl MergeStrategy<Object, Object> for ValueMergeStrategy {
    fn try_merge(mut self, master: &mut Object, candidate: &Object) -> Result<()> {
        if self.depth.is_none() {
            let left = serde_json::from_value::<Squashables>(Value::Object(master.clone()));
            let right = serde_json::from_value::<Squashables>(Value::Object(candidate.clone()));
            match (left, right) {
                (Ok(left), Ok(right)) if left.kind() != right.kind() => {
                    info!(
                        "inferred a replacement: left={}, right={}",
                        left.kind(),
                        right.kind()
                    );
                    let mut candidate = right.into_object()?;
                    if master.contains_key("optional") && !candidate.contains_key("optional") {
                        candidate.insert(
                            "optional".to_string(),
                            master.get("optional").unwrap().clone(),
                        );
                    }
                    *master = candidate;
                    return Ok(());
                }
                (Ok(_), Ok(_)) => info!("matching pair but same kind"),
                (Ok(_), Err(err)) => info!("candidate no match: {}", err),
                (Err(err), Ok(_)) => info!("master no match: {}", err),
                (Err(left), Err(right)) => {
                    info!("no match: master: {}; candidate: {}", left, right)
                }
            };
        }
        if self.replace {
            *master = candidate.clone();
        } else {
            if let Some(depth) = self.depth.as_mut() {
                *depth -= 1;
            }
            for (key, value) in candidate.iter() {
                if let Some(field) = master.get_mut(key) {
                    debug!("try_merge entering '{}'", key);
                    self.try_merge(field, value)
                        .with_context(|| anyhow!("in a field: {}", key))?;
                } else {
                    master.insert(key.clone(), value.clone());
                }
            }
        }
        Ok(())
    }
}

impl MergeStrategy<Value, Value> for ValueMergeStrategy {
    fn try_merge(mut self, master: &mut Value, candidate: &Value) -> Result<()> {
        if let Some(depth) = self.depth.as_ref() {
            if *depth <= 0 {
                self.replace = true;
            }
        }
        debug!(
            "{}::try_merge(master = {}, candidate = {})",
            self, master, candidate
        );
        match (master, candidate) {
            (master, Value::Null) => {
                *master = Value::Null;
            }
            (Value::Bool(master), Value::Bool(candidate)) => *master = *candidate,
            (Value::Number(master), Value::Number(candidate)) => *master = candidate.clone(),
            (Value::String(master), Value::String(candidate)) => *master = candidate.clone(),
            (Value::Array(master), Value::Array(candidate)) => {
                if self.replace {
                    *master = candidate.clone();
                } else {
                    master.extend(candidate.iter().cloned());
                }
            }
            (Value::Object(master), Value::Object(candidate)) => {
                self.try_merge(master, candidate)?;
            }
            (left, right) => {
                return Err(failed!(
                    target: Release,
                    "unexpected value: expected '{}', found '{}'",
                    left.kind(),
                    right.kind()
                ))
            }
        };
        Ok(())
    }
}

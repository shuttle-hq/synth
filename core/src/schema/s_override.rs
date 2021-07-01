use crate::schema::FieldRef;
use crate::schema::{Content, MergeStrategy, ObjectContent, OneOfContent, ValueMergeStrategy};
use crate::Namespace;

use anyhow::Result;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct DefaultOverrideStrategy<'t> {
    pub at: &'t FieldRef,
    pub depth: Option<usize>,
}

pub trait OverrideStrategy {
    fn merge(self, ns: &mut Namespace, value: &Value) -> Result<()>;
    fn delete_from(self, ns: &mut Namespace) -> Result<()>;
}

impl<'t> OverrideStrategy for DefaultOverrideStrategy<'t> {
    fn merge(self, ns: &mut Namespace, value: &Value) -> Result<()> {
        let content = ns.get_s_node_mut(self.at)?;
        let mut serialized = serde_json::to_value(&content)?;
        let strategy = ValueMergeStrategy {
            depth: self.depth.as_ref().map(|depth| *depth as i32),
            replace: false,
        };
        strategy.try_merge(&mut serialized, value)?;

        debug!(
            "merged value={}",
            serde_json::to_string(&serialized).unwrap_or_else(|_| "{unknown}".to_string())
        );

        *content = serde_json::from_value(serialized)?;
        Ok(())
    }

    fn delete_from(self, ns: &mut Namespace) -> Result<()> {
        let parent = self
            .at
            .parent()
            .ok_or_else(|| failed!(target: Release, "attempted to delete a collection"))?;
        // SAFETY: `last` panics only if `fields` not empty, but guaranteed here
        let child = self.at.last();
        let parent_node = ns.get_s_node_mut(&parent)?;
        match parent_node {
        Content::OneOf(OneOfContent { variants }) => {
		let idx: usize = child
		    .parse()
		    .map_err(|err| failed!(target: Release, "invalid index: {}", err)).and_then(|idx| {
			if idx >= variants.len() {
			    Err(failed!(target: Release, "index {} is out of bounds", idx))
			} else {
			    Ok(idx)
			}
		    })?;
		variants.remove(idx);
		Ok(())
	    },
	    Content::Object(ObjectContent { fields }) => {
		fields
		    .remove(&child)
		    .ok_or_else(|| failed!(target: Release, "field '{}' not a member of '{}'", child, parent))?;
		Ok(())
	    },
	    otherwise => Err(failed!(target: Release, "the element referred to by a delete operation needs to be contained inside a 'one_of' or an 'object'; instead we have a '{}'", otherwise))
	}
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::schema::tests::USER_NAMESPACE;
    use crate::schema::{number_content, StringContent};
    use crate::schema::{ArrayContent, FakerContent, NumberContent, RangeStep};

    #[test]
    fn override_leaf() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.address.numbers".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                        "subtype": "u64",
                        "range": {
                "low": 0,
                "high": 100,
                "step": 1
                        }
                    }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::Number(NumberContent::U64(number_content::U64::Range(RangeStep {
                low,
                high,
                step,
            }))) => {
                assert_eq!(*low, 0);
                assert_eq!(*high, 100);
                assert_eq!(*step, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_knows_to_replace_string_subtype() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.address.country".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                "faker": {
                "generator": "name"
                }
                    }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::String(StringContent::Faker(FakerContent {
                generator,
                args: _,
                locales: _,
            })) => {
                assert_eq!(generator.as_str(), "name");
                //assert!(args.is_empty())
                //TODO
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_knows_to_replace_number_subtype() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.user_id".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                        "subtype": "i64",
                "constant": -100
                    }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::Number(NumberContent::I64(number_content::I64::Constant(-100))) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_array() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                        "length": {
                "type": "number",
                "subtype": "u64",
                "constant": 100
                },
                    "content": {
                    "variants": [ {
                        "type": "string",
                        "pattern": "new variant!"
                    } ]
                    }
                        }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::Array(ArrayContent {
                length: box Content::Number(NumberContent::U64(number_content::U64::Constant(100))),
                content,
            }) => match content.as_mut() {
                Content::OneOf(one_of_content) => {
                    let variants = one_of_content.iter().collect::<Vec<_>>();
                    assert_eq!(variants.len(), 3);
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_replace() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: Some(0),
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                "type": "string",
                "pattern": "no friends here"
                    }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::String(StringContent::Pattern(regex)) => {
                assert_eq!(regex.to_string(), "no friends here".to_string());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_schema_no_key() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.i.do.not.exist".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        assert!(strategy.merge(&mut ns, &Value::default()).is_err());
    }

    #[test]
    fn override_schema_mismatched_types() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.address.postcode".parse().unwrap(); // a string in `ns`

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        let res = strategy.merge(
            &mut ns,
            &json!({
                "range": {
                "low": 100
                }
            }),
        );

        assert!(res.is_err());
    }

    #[test]
    fn override_schema_array_variant() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends.content.0".parse().unwrap(); // a string in `ns`

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: Some(0),
        };

        strategy
            .merge(
                &mut ns,
                &json!({
                    "type": "number",
                    "subtype": "u64",
                    "constant": 10
                }),
            )
            .unwrap();

        match ns.get_s_node_mut(&field_ref).unwrap() {
            Content::Number(NumberContent::U64(number_content::U64::Constant(10))) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn override_schema_array_out_of_bounds() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends.10".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        assert!(strategy.merge(&mut ns, &Value::default()).is_err());
    }

    #[test]
    fn override_schema_array_not_an_integer() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends.not_an_integer".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        assert!(strategy.merge(&mut ns, &Value::default()).is_err());
    }

    #[test]
    fn override_delete_from_object() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy.delete_from(&mut ns).unwrap();

        assert!(ns.get_s_node_mut(&field_ref).is_err())
    }

    #[test]
    fn override_delete_from_one_of() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users.friends.content.1".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        strategy.delete_from(&mut ns).unwrap();

        assert!(ns.get_s_node_mut(&field_ref).is_err())
    }

    #[test]
    fn override_cannot_delete_top_level() {
        let mut ns = USER_NAMESPACE.clone();

        let field_ref = "users".parse().unwrap();

        let strategy = DefaultOverrideStrategy {
            at: &field_ref,
            depth: None,
        };

        assert!(strategy.delete_from(&mut ns).is_err());
    }
}

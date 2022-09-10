![allow(clippy::assertions_on_results_states)]
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::schema::{ArrayContent, Content, FieldRef, OneOfContent};
use crate::Namespace;

/// Trait can mutate field into an optional field or can
/// make optional field non-optional
pub trait OptionaliseApi {
    fn optionalise(&mut self, optionalise: Optionalise) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Optionalise {
    at: FieldRef,
    optional: bool,
}

impl OptionaliseApi for Namespace {
    fn optionalise(&mut self, optionalise: Optionalise) -> Result<()> {
        let target = optionalise.at;
        match target.parent() {
            Some(parent) => {
                match self.get_s_node_mut(&parent)? {
                    Content::Object(object_content) |
                    Content::Array(ArrayContent { content: box Content::Object(object_content), .. }) => {
                        let fc = object_content.get_mut(&target.last())?;
                        let owned = std::mem::replace(fc, Content::null());
                        *fc = if optionalise.optional {
                            owned.into_nullable()
                        } else if owned.is_nullable() {
                            match owned {
                                Content::OneOf(OneOfContent { variants }) => {
                                    variants.into_iter().map(|vc| *vc.content).find(|v| !v.is_null()).unwrap()
                                }
                                _ => unreachable!()
                            }
                        } else {
                            owned
                        };
                        Ok(())
                    }
                    otherwise => Err(failed!(target: Release, "Only fields of objects can be optional. But the reference '{}' (whose parent is '{}') is contained in a content node of type '{}'.", target, parent, otherwise.kind()))
                }
            }
            None => {
                Err(failed!(target: Release, "Field '{}' is top-level (meaning it just references a collection). Top-level fields cannot be made optional.", target))
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::schema::tests::USER_NAMESPACE;

    use crate::schema::optionalise::{Optionalise, OptionaliseApi};
    use crate::schema::{Content, FieldRef, ObjectContent};

    #[test]
    fn make_optional() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.address.numbers").unwrap();
        let optionalise = Optionalise {
            at: field.clone(),
            optional: true,
        };
        ns.optionalise(optionalise).unwrap();

        let node = ns.get_s_node_mut(&field.parent().unwrap()).unwrap();
        match node {
            Content::Object(ObjectContent { fields, .. }) => {
                assert!(fields.get("numbers").unwrap().is_nullable())
            }
            _ => panic!("invalid node variant"),
        }
    }

    #[test]
    fn is_reversible() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.address.numbers").unwrap();
        let optionalise = Optionalise {
            at: field.clone(),
            optional: true,
        };
        ns.optionalise(optionalise).unwrap();

        let unoptionalise = Optionalise {
            at: field,
            optional: false,
        };
        ns.optionalise(unoptionalise).unwrap();

        assert_eq!(*USER_NAMESPACE, ns)
    }

    #[test]
    fn is_idempotent() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.address.numbers").unwrap();
        let optionalise = Optionalise {
            at: field,
            optional: true,
        };
        ns.optionalise(optionalise.clone()).unwrap();
        let first_optionalise = ns.clone();
        ns.optionalise(optionalise).unwrap();
        assert_eq!(ns, first_optionalise)
    }

    #[test]
    fn no_such_field() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.address.i-do-not-exist").unwrap();
        let optionalise = Optionalise {
            at: field.clone(),
            optional: true,
        };
        let unoptionalise = Optionalise {
            at: field,
            optional: false,
        };
        assert!(ns.optionalise(optionalise).is_err());
        assert!(ns.optionalise(unoptionalise).is_err());
    }

    #[test]
    fn cannot_make_top_level_optional() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users").unwrap();
        let optionalise = Optionalise {
            at: field.clone(),
            optional: true,
        };
        let unoptionalise = Optionalise {
            at: field,
            optional: false,
        };
        assert!(ns.optionalise(optionalise).is_err());
        assert!(ns.optionalise(unoptionalise).is_err());
    }
}

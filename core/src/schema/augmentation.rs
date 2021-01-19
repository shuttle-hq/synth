use crate::schema::schema::ObjectContent;
use crate::schema::FieldRef;
use crate::{Content, Namespace};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub trait AugmentationApi {
    fn augment(&mut self, augmentation: Augmentation) -> Result<()>;
    fn delete(&mut self, field: FieldRef) -> Result<()>;
}

pub trait AugmentationNode {
    fn augment(&mut self, field: String, content: Content) -> Result<()>;
    fn build_ancestry(&mut self, ancestry: Vec<String>, content: Content) -> Result<()>;
    fn delete_child(&mut self, child: String) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Augmentation {
    field: FieldRef,
    augmentation: Content,
}

impl AugmentationApi for Namespace {
    fn augment(&mut self, augmentation: Augmentation) -> Result<()> {
        if self.contains(&augmentation.field) {
            return Err(
                failed!(target: Release, Conflict => "augmentation failed: field '{}' already exists. Augmentations cannot be performed on existing fields. Try using the 'override' api to change the type of an existing field.", &augmentation.field),
            );
        }

        let parent = augmentation.field
            .parent()
            .ok_or(
                failed!(target: Release, BadRequest => "augmentation failed: augmentations cannot be made at the top level of a collection. You need to at least specify a field, for example '{}.my_field'", augmentation.field.collection)
            )?;

        match self.get_s_node_mut(&parent) {
            Ok(parent_node) => parent_node
                .augment(augmentation.field.last(), augmentation.augmentation)
                .context(format!("at field: {}", parent)),
            Err(_) => self.augment_with_ancestors(
                &parent,
                vec![augmentation.field.last()],
                augmentation.augmentation,
            ),
        }
    }

    fn delete(&mut self, field: FieldRef) -> Result<()> {
        let parent = field
            .parent()
            .ok_or(
                failed!(target: Release, BadRequest => "deletion failed: deletions cannot be made at the top level of a collection. You need to at least specify a field, for example '{}.my_field'", field.collection)
            )?;

        let parent_node = self.get_s_node_mut(&parent)?;
        parent_node
            .delete_child(field.last())
            .context(format!("at field: {}", parent))
    }
}

impl Namespace {
    fn augment_with_ancestors(
        &mut self,
        field: &FieldRef,
        mut visited: Vec<String>,
        content: Content,
    ) -> Result<()> {
        match self.get_s_node_mut(&field) {
            Ok(node) => node.build_ancestry(visited, content),
            Err(_) => match field.parent() {
                Some(parent) => {
                    visited.push(field.last());
                    self.augment_with_ancestors(&parent, visited, content)
                }
                None => {
                    unreachable!("We've reached the top-level which is guaranteed to exist. If you see this error please contact a system administrator.")
                }
            },
        }
    }
}

impl AugmentationNode for Content {
    fn augment(&mut self, field: String, content: Content) -> Result<()> {
        match self {
            Content::Object(object_content) => {
                let map = &mut object_content.0;
                if map.contains_key(&field) {
                    // This may be unreachable but I've left it for good measure
                    return Err(
                        failed!(target: Release, Conflict => "augmentation failed: field '{}' already exists.", field),
                    );
                }
                map.insert(field, content);
                Ok(())
            }
            Content::Array(array_content) => {
                let index: usize = field.parse()?;
                array_content.add_variant_at_index(index, content)?;
                Ok(())
            }
            _ => Err(
                failed!(target: Release, Conflict => "augmentation failed: cannot add augmentation to fields of type '{}'. Fields that can be augmented should be of type 'object' or 'array'.", self),
            ),
        }
    }

    fn build_ancestry(&mut self, mut ancestry: Vec<String>, content: Content) -> Result<()> {
        match self {
            Content::Object(object_content) => match ancestry.len() {
                0 => unreachable!(),
                1 => {
                    object_content
                        .0
                        .insert(ancestry.get(0).unwrap().to_string(), content);
                    Ok(())
                }
                _ => {
                    let mut child = Content::Object(ObjectContent::default());
                    child.build_ancestry(
                        ancestry.as_slice()[0..ancestry.len() - 1].to_vec().clone(),
                        content,
                    )?;
                    object_content.0.insert(ancestry.pop().unwrap(), child);
                    Ok(())
                }
            },
            Content::Array(array_content) => match ancestry.len() {
                0 => unreachable!(),
                1 => {
                    array_content.add_variant(content);
                    Ok(())
                }
                _ => {
                    let mut child = Content::Object(ObjectContent::default());
                    child.build_ancestry(
                        ancestry.as_slice()[0..ancestry.len() - 1].to_vec().clone(),
                        content,
                    )?;
                    let index: usize = ancestry.pop().unwrap().parse()?;
                    array_content.add_variant_at_index(index, child)?;
                    Ok(())
                }
            },
            _ => Err(failed!(target: Release, Conflict =>
                "failed to build augmentation. The requested operation has to go through a node of type '{}' which cannot have children.", self
            )),
        }
    }

    fn delete_child(&mut self, child: String) -> Result<()> {
        match self {
            Content::Array(array_content) => {
                let index: usize = child.parse()?;
                array_content.remove(index)
            }
            Content::Object(object_content) => match object_content.0.remove(&child) {
                None => {
                    use crate::schema::schema;
                    let closest = schema::suggest_closest(object_content.0.keys(), &child)
                        .unwrap_or(", object has no fields".to_string());
                    Err(failed!(target: Release, NotFound => "no such field: '{}'{}", &child, closest))
                }
                Some(_) => Ok(()),
            },
            _ => Err(
                failed!(target: Release, NotFound => "Tried to delete the child of a '{}' which cannot have children", self),
            ),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::schema::tests::USER_NAMESPACE;
    use super::*;

    #[test]
    fn augment() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.some_new_field".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        ns.augment(augmentation.clone()).unwrap();
        assert_eq!(
            ns.get_s_node_mut(&augmentation.field).unwrap(),
            &Content::Null
        )
    }

    #[test]
    fn augment_array() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.friends.2".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        ns.augment(augmentation.clone()).unwrap();
        assert_eq!(
            ns.get_s_node_mut(&augmentation.field).unwrap(),
            &Content::Null
        );
    }

    #[test]
    fn augment_add_top_level_field_is_err() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        assert!(ns.augment(augmentation.clone()).is_err())
    }

    #[test]
    fn augment_existing_field_is_err() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.address.numbers".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        assert!(ns.augment(augmentation).is_err())
    }

    #[test]
    fn augment_on_leaf_is_err() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.last_name.some_field".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        assert!(ns.augment(augmentation).is_err())
    }

    #[test]
    fn augment_build_ancestry() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.some.field.with.many.children".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        ns.augment(augmentation.clone()).unwrap();
        assert_eq!(
            ns.get_s_node_mut(&augmentation.field).unwrap(),
            &Content::Null
        )
    }

    #[test]
    fn augment_build_ancestry_through_leaf_is_err() {
        // Notice, the field 'users.first_name' is a leaf.
        // Therefore the ancestry of the augmentation cannot be built
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.first_name.child.grandchild".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        assert!(ns.augment(augmentation.clone()).is_err())
    }

    #[test]
    fn augment_build_ancestry_through_array() {
        let mut ns = USER_NAMESPACE.clone();
        let augmentation = Augmentation {
            field: FieldRef::new("users.friends.2.some_child".to_string()).unwrap(),
            augmentation: Content::Null,
        };
        ns.augment(augmentation.clone()).unwrap();
        assert_eq!(
            ns.get_s_node_mut(&augmentation.field).unwrap(),
            &Content::Null
        );
    }

    #[test]
    fn delete_node() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.address.postcode".to_string()).unwrap();
        assert!(ns.contains(&field));
        ns.delete(field.clone()).unwrap();
        assert!(!ns.contains(&field));
    }

    #[test]
    fn delete_top_level_is_err() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users".to_string()).unwrap();
        assert!(ns.delete(field.clone()).is_err());
    }

    #[test]
    fn delete_non_existent_is_err() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.i_dont_exist".to_string()).unwrap();
        assert!(ns.delete(field.clone()).is_err());
    }

    #[test]
    fn delete_node_in_array() {
        let mut ns = USER_NAMESPACE.clone();
        let field = FieldRef::new("users.friends.1".to_string()).unwrap();
        assert!(ns.contains(&field));
        ns.delete(field.clone()).unwrap();
        assert!(!ns.contains(&field));
    }
}

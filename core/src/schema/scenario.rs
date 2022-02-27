use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, path::PathBuf};

use crate::{Content, Namespace};

use super::content::{ArrayContent, ObjectContent, SameAsContent};

pub struct Scenario {
    namespace: Namespace,
    scenario: ScenarioNamespace,
    name: String,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
struct ScenarioNamespace {
    #[serde(flatten)]
    collections: BTreeMap<String, ScenarioCollection>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
struct ScenarioCollection {
    #[serde(flatten)]
    fields: BTreeMap<String, Content>,
}

#[derive(Clone)]
enum ContentStatus {
    Included(ContentWrapper),
    Hidden(ContentWrapper),
    Unknown(ContentWrapper),
}

#[derive(Clone)]
enum ContentWrapper {
    Scalar(Content),
    Array {
        length: Box<Content>,
        content: Box<ContentWrapper>,
    },
    Object {
        skip_when_null: bool,
        fields: BTreeMap<String, ContentStatus>,
    },
}

impl ContentStatus {
    fn to_included(self) -> Self {
        match self {
            ContentStatus::Unknown(wrapped) => ContentStatus::Included(wrapped),
            ContentStatus::Hidden(wrapped) => ContentStatus::Included(wrapped),
            status => status,
        }
    }

    fn get_wrapped(self) -> ContentWrapper {
        match self {
            ContentStatus::Included(wrapped)
            | ContentStatus::Hidden(wrapped)
            | ContentStatus::Unknown(wrapped) => wrapped,
        }
    }
}

// We default everything to `unknown` as collections and fields are processed the become `included`
impl From<Content> for ContentStatus {
    fn from(content: Content) -> Self {
        Self::Unknown(content.into())
    }
}

impl From<Content> for ContentWrapper {
    fn from(content: Content) -> Self {
        match content {
            Content::Object(ObjectContent {
                skip_when_null,
                fields,
            }) => Self::Object {
                skip_when_null,
                fields: {
                    let iter = fields.into_iter().map(|(name, field)| (name, field.into()));

                    BTreeMap::from_iter(iter)
                },
            },
            Content::Array(ArrayContent { length, content }) => Self::Array {
                length,
                content: Box::new(Box::into_inner(content).into()),
            },
            scalar => Self::Scalar(scalar),
        }
    }
}

// Everything that is still unknown should not be in the output.
impl From<ContentStatus> for Option<Content> {
    fn from(status: ContentStatus) -> Self {
        match status {
            ContentStatus::Unknown(_) => None,
            ContentStatus::Hidden(content) => {
                let content: Content = content.into();
                let content = content.into_hidden();
                Some(content)
            }
            ContentStatus::Included(content) => Some(content.into()),
        }
    }
}

impl From<ContentWrapper> for Content {
    fn from(wrapper: ContentWrapper) -> Self {
        match wrapper {
            ContentWrapper::Scalar(content) => content,
            ContentWrapper::Array { length, content } => Content::Array(ArrayContent {
                length,
                content: Box::new(Box::into_inner(content).into()),
            }),
            ContentWrapper::Object {
                skip_when_null,
                fields,
            } => Content::Object(ObjectContent {
                skip_when_null,
                fields: {
                    let iter = fields.into_iter().filter_map(|(name, field)| {
                        if let Some(content) = field.into() {
                            Some((name, content))
                        } else {
                            None
                        }
                    });

                    BTreeMap::from_iter(iter)
                },
            }),
        }
    }
}

impl PartialEq<Content> for ContentWrapper {
    fn eq(&self, other: &Content) -> bool {
        match (self, other) {
            (ContentWrapper::Scalar(Content::Null(orig)), Content::Null(over)) => orig == over,
            (ContentWrapper::Scalar(Content::Bool(orig)), Content::Bool(over)) => orig == over,
            (ContentWrapper::Scalar(Content::Number(orig)), Content::Number(over)) => orig == over,
            (ContentWrapper::Scalar(Content::String(orig)), Content::String(over)) => orig == over,
            (ContentWrapper::Scalar(Content::DateTime(orig)), Content::DateTime(over)) => {
                orig == over
            }
            (ContentWrapper::Scalar(Content::OneOf(orig)), Content::OneOf(over)) => orig == over,
            (ContentWrapper::Scalar(Content::SameAs(orig)), Content::SameAs(over)) => orig == over,
            (ContentWrapper::Scalar(Content::Series(orig)), Content::Series(over)) => orig == over,
            (ContentWrapper::Scalar(Content::Datasource(orig)), Content::Datasource(over)) => {
                orig == over
            }
            (ContentWrapper::Scalar(Content::Hidden(orig)), Content::Hidden(over)) => orig == over,
            (ContentWrapper::Scalar(Content::Unique(orig)), Content::Unique(over)) => orig == over,
            (
                ContentWrapper::Array { content, length },
                Content::Array(ArrayContent {
                    content: overwrite_content,
                    length: overwrite_length,
                }),
            ) => length == overwrite_length && content.as_ref() == overwrite_content.as_ref(),
            (
                ContentWrapper::Object {
                    skip_when_null,
                    fields,
                },
                Content::Object(ObjectContent {
                    fields: overwrite_fields,
                    skip_when_null: overwrite_skip_when_null,
                }),
            ) => {
                fields.len() == overwrite_fields.len()
                    && skip_when_null == overwrite_skip_when_null
                    && fields.iter().all(|(name, content)| {
                        if let Some(over) = overwrite_fields.get(name) {
                            if let ContentStatus::Unknown(orig) = content {
                                orig == over
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
            }
            _ => false,
        }
    }
}

impl Scenario {
    pub fn new(namespace: Namespace, namespace_path: PathBuf, scenario: &str) -> Result<Self> {
        let scenario_path = namespace_path
            .join("scenarios")
            .join(scenario)
            .with_extension("json");

        if !scenario_path.exists() {
            return Err(anyhow!(
                "could not find scenario at {}",
                scenario_path.display()
            ));
        }

        let scenario_content = std::fs::read_to_string(scenario_path.clone())?;
        debug!("found scenario:\n{}", scenario);
        let scenario_namespace = serde_json::from_str(&scenario_content).context(anyhow!(
            "Failed to parse scenario '{}'",
            scenario_path.display()
        ))?;

        Ok(Self {
            namespace,
            scenario: scenario_namespace,
            name: scenario.to_string(),
        })
    }

    pub fn build(self) -> Result<Namespace> {
        self.has_extra_collections()
            .context(anyhow!("failed to build scenario '{}'", self.name))?;
        let scenario = self.scenario;
        let iter = self
            .namespace
            .into_iter()
            .map(|(collection, fields)| (collection, fields.into()));
        let mut original_collections: BTreeMap<String, ContentStatus> = BTreeMap::from_iter(iter);

        Self::trim_fields(&mut original_collections, scenario.collections)?;

        let included_refs = Self::find_included_references(&original_collections);
        Self::make_missing_references_hidden(&mut original_collections, included_refs);

        let mut namespace = Namespace::new();
        for (collection, fields) in original_collections {
            if let Some(content) = fields.into() {
                namespace.put_collection(collection, content)?;
            }
        }

        Ok(namespace)
    }

    fn find_included_references(collections: &BTreeMap<String, ContentStatus>) -> Vec<Vec<String>> {
        let mut refs = Vec::new();

        for status in collections.values() {
            match status {
                ContentStatus::Included(wrapper) => {
                    refs.append(&mut Self::find_included_references_wrapper(wrapper))
                }
                _ => continue,
            }
        }

        refs
    }

    fn find_included_references_wrapper(wrapper: &ContentWrapper) -> Vec<Vec<String>> {
        let mut refs = Vec::new();

        match wrapper {
            ContentWrapper::Scalar(Content::SameAs(SameAsContent { ref_ })) => {
                let mut r#ref = vec![ref_.collection.clone()];
                r#ref.append(&mut ref_.fields.clone());
                refs.push(r#ref);
            }
            ContentWrapper::Array { content, .. } => {
                refs.append(&mut Self::find_included_references_wrapper(content))
            }
            ContentWrapper::Object { fields, .. } => {
                refs.append(&mut Self::find_included_references(fields))
            }
            _ => {}
        }

        refs
    }

    fn make_missing_references_hidden(
        content: &mut BTreeMap<std::string::String, ContentStatus>,
        refs: Vec<Vec<String>>,
    ) {
        let mut refs = refs;

        while let Some(mut r#ref) = refs.pop() {
            r#ref.reverse();
            if let Some(extra) = Self::make_reference_hidden(content, r#ref) {
                refs.push(extra);
            }
        }
    }

    fn make_reference_hidden(
        content: &mut BTreeMap<std::string::String, ContentStatus>,
        r#ref: Vec<String>,
    ) -> Option<Vec<String>> {
        let mut r#ref = r#ref;

        if let Some(path) = r#ref.pop() {
            if let Some(field) = content.remove(&path) {
                let (field, extra) = match field {
                    ContentStatus::Included(mut wrapped) => {
                        let extra = Self::make_reference_hidden_wrapped(&mut wrapped, r#ref);
                        (ContentStatus::Included(wrapped), extra)
                    }
                    ContentStatus::Unknown(mut wrapped) => {
                        let extra = Self::make_reference_include_wrapped(&mut wrapped, r#ref);
                        (ContentStatus::Hidden(wrapped), extra)
                    }
                    hidden => (hidden, None),
                };

                content.insert(path, field);

                return extra;
            }
        }

        None
    }

    fn make_reference_hidden_wrapped(
        wrapped: &mut ContentWrapper,
        r#ref: Vec<String>,
    ) -> Option<Vec<String>> {
        let mut r#ref = r#ref;
        match wrapped {
            ContentWrapper::Scalar(Content::SameAs(SameAsContent { ref_ })) => {
                let mut extra = vec![ref_.collection.clone()];
                extra.append(&mut ref_.fields.clone());
                Some(extra)
            }
            ContentWrapper::Scalar(_) => None,
            ContentWrapper::Array { content, .. } => {
                if let Some(path) = r#ref.pop() {
                    if path.as_str() == "content" {
                        return Self::make_reference_hidden_wrapped(content, r#ref);
                    }
                }

                None
            }
            ContentWrapper::Object { fields, .. } => Self::make_reference_hidden(fields, r#ref),
        }
    }

    fn make_reference_include(
        content: &mut BTreeMap<std::string::String, ContentStatus>,
        r#ref: Vec<String>,
    ) -> Option<Vec<String>> {
        let mut r#ref = r#ref;

        if let Some(path) = r#ref.pop() {
            if let Some(field) = content.remove(&path) {
                let (field, extra) = match field {
                    ContentStatus::Unknown(mut wrapped) => {
                        let extra = Self::make_reference_include_wrapped(&mut wrapped, r#ref);
                        (ContentStatus::Included(wrapped), extra)
                    }
                    _ => unreachable!("the inside of an unknown is always unknown too"),
                };

                content.insert(path, field);

                return extra;
            }
        }

        None
    }

    fn make_reference_include_wrapped(
        wrapped: &mut ContentWrapper,
        r#ref: Vec<String>,
    ) -> Option<Vec<String>> {
        let mut r#ref = r#ref;
        match wrapped {
            ContentWrapper::Scalar(Content::SameAs(SameAsContent { ref_ })) => {
                let mut extra = vec![ref_.collection.clone()];
                extra.append(&mut ref_.fields.clone());
                Some(extra)
            }
            ContentWrapper::Scalar(_) => None,
            ContentWrapper::Array { content, .. } => {
                if let Some(path) = r#ref.pop() {
                    if path.as_str() == "content" {
                        return Self::make_reference_include_wrapped(content, r#ref);
                    }
                }

                None
            }
            ContentWrapper::Object { fields, .. } => Self::make_reference_include(fields, r#ref),
        }
    }

    fn has_extra_collections(&self) -> Result<()> {
        let original_collections: Vec<_> = self.namespace.keys().collect();

        let extra_collections: Vec<_> = self
            .scenario
            .collections
            .keys()
            .filter(|c| !original_collections.contains(&c.as_str()))
            .collect();

        if !extra_collections.is_empty() {
            let extra_collections = extra_collections
                .into_iter()
                .map(|e| format!("- {}", e))
                .collect::<Vec<String>>()
                .join("\n");

            return Err(anyhow!(
                "the namespace does not contain the following collection(s):\n{}",
                extra_collections
            ));
        }

        Ok(())
    }

    fn trim_fields(
        original: &mut BTreeMap<String, ContentStatus>,
        overwrite: BTreeMap<String, ScenarioCollection>,
    ) -> Result<()> {
        for (name, overwrite_collection) in overwrite {
            // Safe to unwrap since we already confirmed it exists in `trim_namespace_collections`
            let original_collection = original.remove(&name).unwrap();

            if overwrite_collection.fields.is_empty() {
                // This is an include only field
                let original_collection = original_collection.to_included();
                original.insert(name, original_collection);
                continue;
            }

            let mut wrapped = original_collection.get_wrapped();

            Self::trim_collection_fields(&mut wrapped, &overwrite_collection.fields)
                .context(anyhow!("failed to trim collection '{}'", name))?;

            original.insert(name, ContentStatus::Included(wrapped));
        }

        Ok(())
    }

    fn trim_collection_fields(
        original: &mut ContentWrapper,
        overwrites: &BTreeMap<String, Content>,
    ) -> Result<()> {
        match original {
            ContentWrapper::Object { fields, .. } => {
                Self::merge_object_content(fields, overwrites)?
            }
            ContentWrapper::Array { content, .. } => {
                Self::trim_collection_fields(content.as_mut(), overwrites)?;
            }
            _ => return Err(anyhow!("cannot select fields to include from a non-object")),
        };

        Ok(())
    }

    fn merge_field(original: &mut ContentWrapper, overwrite: &Content) -> Result<()> {
        // We check if types are the same first to find redundant overwrites
        if original == overwrite {
            return Self::same_err();
        }

        match (original, overwrite) {
            (
                ContentWrapper::Array { content, length },
                Content::Array(ArrayContent {
                    content: overwrite_content,
                    length: overwrite_length,
                }),
            ) => {
                *content = Box::new(overwrite_content.as_ref().clone().into());
                *length = overwrite_length.clone();
            }
            (
                ContentWrapper::Object {
                    skip_when_null,
                    fields,
                },
                Content::Object(ObjectContent {
                    fields: overwrite_fields,
                    skip_when_null: overwrite_skip_when_null,
                }),
            ) => {
                Self::merge_object_content(fields, overwrite_fields)?;
                *skip_when_null = *overwrite_skip_when_null;
            }
            (original, overwrite) => *original = overwrite.clone().into(),
        }

        Ok(())
    }

    fn merge_object_content(
        original: &mut BTreeMap<String, ContentStatus>,
        overwrites: &BTreeMap<String, Content>,
    ) -> Result<()> {
        let mut originals_len = original.len() as isize;

        for (name, overwrite_content) in overwrites {
            if let Some(original_field) = original.remove(name) {
                // If this just an include
                if let Content::Empty(_) = overwrite_content {
                    let original_field = original_field.to_included();
                    original.insert(name.to_string(), original_field);
                    originals_len -= 1;
                    continue;
                }
                debug!("merging field '{}'", name);

                let mut wrapped = original_field.get_wrapped();
                Self::merge_field(&mut wrapped, overwrite_content)
                    .context(anyhow!("failed to overwrite field '{}'", name))?;
                original.insert(name.to_string(), ContentStatus::Included(wrapped));
                originals_len = -1;
            } else {
                // Cannot add include only fields
                if let Content::Empty(_) = overwrite_content {
                    return Err(anyhow!(
                        "'{}' is not a field on the object, therefore it cannot be included",
                        name
                    ));
                }

                original.insert(
                    name.to_string(),
                    ContentStatus::Included(overwrite_content.clone().into()),
                );
                originals_len = -1;
            }
        }

        if originals_len == 0 {
            return Err(anyhow!("all fields from object are included as is with no overwrites. Consider making the parent an include instead."));
        }

        Ok(())
    }

    fn same_err() -> Result<()> {
        Err(anyhow!("overwrite is same as original"))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    use tempfile::tempdir;

    use super::Scenario;

    macro_rules! namespace {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<crate::Namespace>(serde_json::json!($($inner)*))
                .expect("could not deserialize into a namespace")
        }
    }

    macro_rules! scenario {
        {
            $($inner:tt)*
        } => {
            serde_json::from_value::<super::ScenarioNamespace>(serde_json::json!($($inner)*))
                .expect("could not deserialize into a scenario namespace")
        }
    }

    #[test]
    #[should_panic(expected = "could not find scenario")]
    fn new_missing() {
        let path = tempdir().unwrap().path().into();
        Scenario::new(Default::default(), path, "missing").unwrap();
    }

    #[test]
    fn new_exist() -> Result<()> {
        let path: PathBuf = tempdir()?.path().into();

        let scenario_dir = path.join("scenarios");
        fs::create_dir_all(&scenario_dir)?;

        let scenario_path = scenario_dir.join("dummy").with_extension("json");
        let mut file = File::create(scenario_path)?;
        write!(file, r#"{{ "collection": {{}} }}"#)?;

        let scenario = Scenario::new(Default::default(), path, "dummy").unwrap();

        let expected = scenario!({"collection": {}});

        assert_eq!(scenario.scenario, expected);

        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "the namespace does not contain the following collection(s):\n    - extra"
    )]
    fn build_extra_collection() {
        let scenario = Scenario {
            namespace: Default::default(),
            scenario: scenario!({"extra": {}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_filter_collections() {
        let scenario = Scenario {
            namespace: namespace!({"collection1": {}, "collection2": {}}),
            scenario: scenario!({"collection1": {}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({"collection1": {}});

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_filter_fields() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection1": {
                "type": "object",
                "nully": {"type": "null"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_filter_fields_array() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "array",
                    "length": 5,
                    "content": {
                        "type": "object",
                        "nully": {"type": "null"},
                        "stringy": {"type": "string", "pattern": "test"}
                    }
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection1": {
                "type": "array",
                "length": 5,
                "content": {
                    "type": "object",
                    "nully": {"type": "null"},
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "'null' is not a field on the object")]
    fn build_filter_extra_field_include() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"null": {}}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_filter_extra_field() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                },
                "collection2": {}
            }),
            scenario: scenario!({"collection1": {"number": {"type": "number", "constant": 4}}}),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection1": {
                "type": "object",
                "number": {
                    "type": "number",
                    "constant": 4
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "cannot select fields to include from a non-object")]
    fn build_filter_field_scalar() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection1": {
                    "type": "null"
                },
            }),
            scenario: scenario!({"collection1": {"nully": {}}}),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_types() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "stringy": {"type": "string", "pattern": "test"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully": {"type": "string", "pattern": "test"},
                    "stringy": {"type": "null"}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "nully": {"type": "string", "pattern": "test"},
                "stringy": {"type": "null"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_null() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully": {"type": "null"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully": {"type": "null"}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_bool() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "bool_constant": {"type": "bool", "constant": true},
                    "bool_frequency": {"type": "bool", "frequency": 0.5},
                    "bool_subtype": {"type": "bool", "constant": true}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "bool_constant": {"type": "bool", "constant": false},
                    "bool_frequency": {"type": "bool", "frequency": 0.3},
                    "bool_subtype": {"type": "bool", "frequency": 0.8}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "bool_constant": {"type": "bool", "constant": false},
                "bool_frequency": {"type": "bool", "frequency": 0.3},
                "bool_subtype": {"type": "bool", "frequency": 0.8}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_bool_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "bool", "constant": true}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "bool", "constant": true}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_number() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 2},
                    "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 0, "high": 5, "step": 1}},
                    "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 3}},
                    "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 2},
                    "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 0, "high": 5, "step": 1}},
                    "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 3}},
                    "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -2},
                    "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -3, "high": 5, "step": 1}},
                    "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -3}},
                    "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -2},
                    "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -3, "high": 5, "step": 1}},
                    "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -3}},
                    "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 3.2},
                    "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 3.1, "high": 5.3, "step": 0.1}},
                    "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 4.2},
                    "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 34.2, "high": 56.3, "step": 0.3}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 3},
                    "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 2, "high": 10, "step": 3}},
                    "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 7}},
                    "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 8},
                    "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 4, "high": 10, "step": 2}},
                    "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 1}},
                    "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -7},
                    "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -8, "high": 7, "step": 2}},
                    "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -2}},
                    "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -9},
                    "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -8, "high": 7, "step": 2}},
                    "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -6}},
                    "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 6.2},
                    "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 4.2, "high": 8.3, "step": 0.7}},
                    "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 6.3},
                    "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 3.2, "high": 5.3, "step": 0.04}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "number_u32_constant": {"type": "number", "subtype": "u32", "constant": 3},
                "number_u32_range": {"type": "number", "subtype": "u32", "range": {"low": 2, "high": 10, "step": 3}},
                "number_u32_id": {"type": "number", "subtype": "u32", "id": {"start_at": 7}},
                "number_u64_constant": {"type": "number", "subtype": "u64", "constant": 8},
                "number_u64_range": {"type": "number", "subtype": "u64", "range": {"low": 4, "high": 10, "step": 2}},
                "number_u64_id": {"type": "number", "subtype": "u64", "id": {"start_at": 1}},
                "number_i32_constant": {"type": "number", "subtype": "i32", "constant": -7},
                "number_i32_range": {"type": "number", "subtype": "i32", "range": {"low": -8, "high": 7, "step": 2}},
                "number_i32_id": {"type": "number", "subtype": "i32", "id": {"start_at": -2}},
                "number_i64_constant": {"type": "number", "subtype": "i64", "constant": -9},
                "number_i64_range": {"type": "number", "subtype": "i64", "range": {"low": -8, "high": 7, "step": 2}},
                "number_i64_id": {"type": "number", "subtype": "i64", "id": {"start_at": -6}},
                "number_f32_constant": {"type": "number", "subtype": "f32", "constant": 6.2},
                "number_f32_range": {"type": "number", "subtype": "f32", "range": {"low": 4.2, "high": 8.3, "step": 0.7}},
                "number_f64_constant": {"type": "number", "subtype": "f64", "constant": 6.3},
                "number_f64_range": {"type": "number", "subtype": "f64", "range": {"low": 3.2, "high": 5.3, "step": 0.04}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_number_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "number", "constant": 2}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "number", "constant": 2}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_string() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "string_pattern": {"type": "string", "pattern": "(m|f)"},
                    "string_format": {"type": "string", "format": {"format": "the lucky number is {number}", "arguments": {"number": 5}}},
                    "string_faker": {"type": "string", "faker": {"generator": "address"}},
                    "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 3}},
                    "string_truncated": {"type": "string", "truncated": {"length": 4, "content": {"type": "string", "pattern": "[a-z]{30}"}}},
                    "string_sliced": {"type": "string", "sliced": {"slice": "4:8", "content": {"type": "string", "pattern": "[a-z]{30}"}}},
                    "string_constant": {"type": "string", "constant": "hello world"},
                    "string_categorical": {"type": "string", "categorical": {"hello": 4, "world": 3}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "string_pattern": {"type": "string", "pattern": "(male|female)"},
                    "string_format": {"type": "string", "format": {"format": "{number} was guessed", "arguments": {"number": 2}}},
                    "string_faker": {"type": "string", "faker": {"generator": "first_name"}},
                    "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 7}},
                    "string_truncated": {"type": "string", "truncated": {"length": 8, "content": {"type": "string", "pattern": "[A-Z]{35}"}}},
                    "string_sliced": {"type": "string", "sliced": {"slice": "25:30", "content": {"type": "string", "pattern": "[A-Z]{63}"}}},
                    "string_constant": {"type": "string", "constant": "bye world"},
                    "string_categorical": {"type": "string", "categorical": {"bye": 8, "world": 6}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "string_pattern": {"type": "string", "pattern": "(male|female)"},
                "string_format": {"type": "string", "format": {"format": "{number} was guessed", "arguments": {"number": 2}}},
                "string_faker": {"type": "string", "faker": {"generator": "first_name"}},
                "string_serialized": {"type": "string", "serialized": {"serializer": "json", "content": 7}},
                "string_truncated": {"type": "string", "truncated": {"length": 8, "content": {"type": "string", "pattern": "[A-Z]{35}"}}},
                "string_sliced": {"type": "string", "sliced": {"slice": "25:30", "content": {"type": "string", "pattern": "[A-Z]{63}"}}},
                "string_constant": {"type": "string", "constant": "bye world"},
                "string_categorical": {"type": "string", "categorical": {"bye": 8, "world": 6}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_string_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "string_uuid": {"type": "string", "uuid": {}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "string_uuid": {"type": "string", "uuid": {}}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_date_time() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"},
                    "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "13:34:34", "end": "14:32:53"},
                    "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-03-23T13:34:34", "end": "2022-04-30T3:32:53"},
                    "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-03-23T13:34:34+0100", "end": "2022-04-30T3:32:53+0100"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-01-31", "end": "2022-02-23"},
                    "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "22:32:35", "end": "23:34:35"},
                    "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-02-25T11:49:39", "end": "2022-03-12T12:39:28"},
                    "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-02-25T11:49:39+0000", "end": "2022-03-12T12:39:28+0000"}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "date_time_naive_date": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-01-31", "end": "2022-02-23"},
                "date_time_naive_time": {"type": "date_time", "subtype": "naive_time", "format": "%H:%M:%S", "begin": "22:32:35", "end": "23:34:35"},
                "date_time_naive_date_time": {"type": "date_time", "subtype": "naive_date_time", "format": "%Y-%m-%dT%H:%M:%S", "begin": "2022-02-25T11:49:39", "end": "2022-03-12T12:39:28"},
                "date_time_date_time": {"type": "date_time", "subtype": "date_time", "format": "%Y-%m-%dT%H:%M:%S%z", "begin": "2022-02-25T11:49:39+0000", "end": "2022-03-12T12:39:28+0000"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_date_time_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "date_time", "subtype": "naive_date", "format": "%Y-%m-%d", "begin": "2022-02-15", "end": "2022-02-19"}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_one_of() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "one_of": {"type": "one_of", "variants": [{"weight": 0.5, "type": "string", "pattern": "m|f"}, {"weight": 0.5, "type": "null"}]}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "one_of": {"type": "one_of", "variants": [{"weight": 0.8, "type": "string", "pattern": "m|f"}, {"weight": 0.2, "type": "null"}]}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "one_of": {"type": "one_of", "variants": [{"weight": 0.8, "type": "string", "pattern": "m|f"}, {"weight": 0.2, "type": "null"}]}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_one_of_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "one_of", "variants": [{"weight": 0.5, "type": "string", "pattern": "m|f"}, {"weight": 0.5, "type": "null"}]}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "one_of", "variants": [{"weight": 0.5, "type": "null"}, {"weight": 0.5, "type": "string", "pattern": "m|f"}]}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_same_as() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same_as": {"type": "same_as", "ref": "here.content.field"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same_as": {"type": "same_as", "ref": "other.content.name"}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "same_as": {"type": "same_as", "ref": "other.content.name"}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_same_as_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "same_as", "ref": "here.content.field"}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "same_as", "ref": "here.content.field"}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_unique() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "u": {"type": "unique", "content": { "type": "null" }}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "u": {"type": "unique", "content": { "type": "string", "pattern": "f|m" }}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "u": {"type": "unique", "content": { "type": "string", "pattern": "f|m" }}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_unique_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "unique", "content": { "type": "null" }}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "null", "unique": true}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_series() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "series_incrementing": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "incrementing": {"start": "2022-03-20 3:34:00", "increment": "1m"}},
                    "series_poisson": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "poisson": {"start": "2022-03-18 8:29:00", "rate": "1m"}},
                    "series_cyclical": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "cyclical": {"start": "2022-03-23 9:23:00", "period": "1d", "min_rate": "30m", "max_rate": "1m"}},
                    "series_zip": {"type": "series", "zip": {"series": [{"poisson": {"start": "2022-03-18 8:29:00", "rate": "1m"}}, {"incrementing": {"start": "2022-03-20 3:34:00", "increment": "1m"}}]}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "series_incrementing": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "incrementing": {"start": "2022-03-28 2:39:00", "increment": "5m"}},
                    "series_poisson": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "poisson": {"start": "2022-03-29 23:28:00", "rate": "1m"}},
                    "series_cyclical": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "cyclical": {"start": "2022-03-22 23:12:00", "period": "1d", "min_rate": "10m", "max_rate": "30s"}},
                    "series_zip": {"type": "series", "zip": {"series": [{"poisson": {"start": "2022-03-29 8:29:00", "rate": "1m"}}, {"incrementing": {"start": "2022-03-29 3:34:00", "increment": "1m"}}]}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "series_incrementing": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "incrementing": {"start": "2022-03-28 2:39:00", "increment": "5m"}},
                "series_poisson": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "poisson": {"start": "2022-03-29 23:28:00", "rate": "1m"}},
                "series_cyclical": {"type": "series", "format": "%Y-%m-%d %H:%M:%S", "cyclical": {"start": "2022-03-22 23:12:00", "period": "1d", "min_rate": "10m", "max_rate": "30s"}},
                "series_zip": {"type": "series", "zip": {"series": [{"poisson": {"start": "2022-03-29 8:29:00", "rate": "1m"}}, {"incrementing": {"start": "2022-03-29 3:34:00", "increment": "1m"}}]}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_series_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "series", "zip": {"series": [{"poisson": {"start": "2022-03-29 8:29:00", "rate": "1m"}}, {"incrementing": {"start": "2022-03-29 3:34:00", "increment": "1m"}}]}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "series", "zip": {"series": [{"incrementing": {"start": "2022-03-29 3:34:00", "increment": "1m"}}, {"poisson": {"start": "2022-03-29 8:29:00", "rate": "1m"}}]}}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_datasource() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "datasource": {"type": "datasource", "path": "json:users.json", "cycle": false}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "datasource": {"type": "datasource", "path": "json:people.json", "cycle": true}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "datasource": {"type": "datasource", "path": "json:people.json", "cycle": true}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_datasource_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "datasource": {"type": "datasource", "path": "json:users.json", "cycle": false}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "datasource": {"type": "datasource", "path": "json:users.json", "cycle": false}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_hidden() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "h": {"type": "hidden", "content": { "type": "null" }}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "h": {"type": "hidden", "content": { "type": "string", "pattern": "f|m" }}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "h": {"type": "hidden", "content": { "type": "string", "pattern": "f|m" }}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_hidden_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "same": {"type": "hidden", "content": { "type": "null" }}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "same": {"type": "null", "hidden": true}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_array() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "array": {"type": "array", "length": 5, "content": {"type": "null"}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "array": {"type": "array", "length": 5, "content": {"type": "string", "constant": "hello"}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "array": {"type": "array", "length": 5, "content": {"type": "string", "constant": "hello"}}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_array_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "array": {"type": "array", "length": 5, "content": {"type": "null"}}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "array": {"type": "array", "length": 5, "content": {"type": "null"}}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_overwrite_object() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "object": {"type": "object", "nully": {"type": "null"}, "other": 7}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "object": {"type": "object", "nully": {"type": "string", "constant": "hello"}, "other": {}}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "object": {"type": "object", "nully": {"type": "string", "constant": "hello"}, "other": 7}
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "overwrite is same as original")]
    fn build_overwrite_object_same() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "object": {"type": "object", "nully": {"type": "null"}, "other": 7}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "object": {"type": "object", "nully": {"type": "null"}, "other": 7}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    #[should_panic(expected = "all fields from object are included as is")]
    fn build_overwrite_object_same_includes_all() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "object": {"type": "object", "nully": {"type": "null"}, "other": 7}
                }
            }),
            scenario: scenario!({
                "collection": {
                    "object": {"type": "object", "nully": {}, "other": {}}
                }
            }),
            name: "test".to_string(),
        };

        scenario.build().unwrap();
    }

    #[test]
    fn build_complex() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "user": {
                        "type": "object",
                        "name": {"type": "string", "faker": {"generator": "first_name"}},
                        "age": {"type": "number", "range": {"low": 20, "high": 60}},
                        "contact": {
                            "type": "object",
                            "phone": {"type": "string", "faker": {"generator": "phone_number"}},
                            "email": {"type": "string", "faker": {"generator": "safe_email"}}
                        }
                    }
                }
            }),
            scenario: scenario!({
                "collection": {
                    "user": {
                        "type": "object",
                        "name": {},
                        "contact": {
                            "type": "object",
                            "email": {"type": "string", "faker": {"generator": "free_email"}}
                        },
                        "can_contact": {"type": "bool", "frequency": 0.8}
                    }
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "user": {
                    "type": "object",
                    "name": {"type": "string", "faker": {"generator": "first_name"}},
                    "contact": {
                        "type": "object",
                        "email": {"type": "string", "faker": {"generator": "free_email"}}
                    },
                    "can_contact": {"type": "bool", "frequency": 0.8}
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_missing_references() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully": {"type": "null"},
                    "nully_ref": "@collection.nully",
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully_ref": {}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "nully": {"type": "null", "hidden": true},
                "nully_ref": "@collection.nully",
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_missing_references_chain() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully_ref2": "@collection.nully_ref",
                    "nully_ref": "@collection2.content.nully",
                },
                "collection2": {
                    "type": "array",
                    "length": 3,
                    "content": {
                        "type": "object",
                        "nully": {"type": "null"},
                    }
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully_ref2": {}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "nully_ref2": "@collection.nully_ref",
                "nully_ref":{
                    "type": "same_as",
                    "ref": "collection2.content.nully",
                    "hidden": true
                },
            },
            "collection2": {
                "type": "array",
                "length": 3,
                "hidden": true,
                "content": {
                    "type": "object",
                    "nully": {"type": "null"},
                }
            }
        });

        assert_eq!(actual, expected);
    }

    #[test]
    fn build_missing_references_double() {
        let scenario = Scenario {
            namespace: namespace!({
                "collection": {
                    "type": "object",
                    "nully_ref": "@collection2.content.nully",
                },
                "collection2": {
                    "type": "array",
                    "length": 3,
                    "content": {
                        "type": "object",
                        "nully": {"type": "null"},
                    }
                }
            }),
            scenario: scenario!({
                "collection": {
                    "nully_ref2": "@collection2.content.nully",
                    "nully_ref": {}
                }
            }),
            name: "test".to_string(),
        };

        let actual = scenario.build().unwrap();
        let expected = namespace!({
            "collection": {
                "type": "object",
                "nully_ref2": "@collection2.content.nully",
                "nully_ref": "@collection2.content.nully",
            },
            "collection2": {
                "type": "array",
                "length": 3,
                "hidden": true,
                "content": {
                    "type": "object",
                    "nully": {"type": "null"},
                }
            }
        });

        assert_eq!(actual, expected);
    }
}

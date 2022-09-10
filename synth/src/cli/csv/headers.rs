use synth_core::schema::content::{ArrayContent, ObjectContent, SameAsContent};
use synth_core::{Content, Namespace};

use super::determine_content_array_max_length;

use std::fmt;

use anyhow::Result;

use regex::Regex;

pub struct CsvHeaders(Vec<CsvHeader>);

impl CsvHeaders {
    /// Flattern a `Content` instance into a set of CSV headers. The `content` parameter should correspond to the inner
    /// content value inside of the outer most array generator in a schema.
    pub fn from_content(content: &Content, namespace: &Namespace) -> Result<Self> {
        match content {
            Content::Object(obj) => parse_object_to_headers(None, obj, namespace),
            Content::Array(array) => parse_array_to_headers(None, array, namespace),
            Content::OneOf(_) => parse_one_of_to_headers(
                CsvHeader::ObjectProperty {
                    key: "one_of".to_string(),
                    parent: None,
                },
                content,
                namespace,
            ),
            Content::SameAs(same_as) => parse_same_as_to_headers(
                CsvHeader::ObjectProperty {
                    key: "same_as".to_string(),
                    parent: None,
                },
                same_as,
                namespace,
            ),
            Content::Unique(unique) => parse_content_to_headers(
                CsvHeader::ObjectProperty {
                    key: "unique".to_string(),
                    parent: None,
                },
                &unique.content,
                namespace,
            ),
            _ => Ok(vec![CsvHeader::ObjectProperty {
                key: "value".to_string(),
                parent: None,
            }]),
        }
        .map(CsvHeaders)
    }

    pub fn from_csv_header_record(record: &csv::StringRecord) -> Result<Self> {
        record
            .into_iter()
            .map(CsvHeader::from_csv_str)
            .collect::<Result<Vec<CsvHeader>>>()
            .map(CsvHeaders)
    }

    pub fn to_csv_record(&self) -> csv::StringRecord {
        csv::StringRecord::from(
            self.0
                .iter()
                .map(CsvHeader::to_string)
                .collect::<Vec<String>>(),
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = &CsvHeader> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CsvHeader {
    ArrayElement {
        parent: Option<Box<CsvHeader>>,
        index: usize,
        max_length: usize,
    },
    ObjectProperty {
        parent: Option<Box<CsvHeader>>,
        key: String,
    },
}

lazy_static::lazy_static! {
    static ref ARRAY_INDEX_REGEX: Regex = Regex::new(r"\A\[([0-9]+)\]").unwrap();
}

impl CsvHeader {
    fn from_csv_str(s: &str) -> Result<Self> {
        let mut s_index = 0;
        let mut header = None;

        while s_index < s.len() {
            let substr = &s[s_index..];
            let search = ARRAY_INDEX_REGEX.find(substr);

            header = if let Some(found) = search {
                let found_str = found.as_str();

                s_index += found_str.len();

                let index = found_str[1..found_str.len() - 1].parse().unwrap();

                Some(CsvHeader::ArrayElement {
                    parent: header.map(Box::new),
                    index,
                    max_length: 0,
                })
            } else {
                let find_index = substr
                    .find(|c| c == '.' || c == '[')
                    .unwrap_or(substr.len());

                let key = &substr[..find_index];

                if key.is_empty() {
                    return Err(anyhow!(
                        "Invalid CSV header '{}' - cannot have an empty object property name.",
                        s
                    ));
                }

                s_index += key.len();

                Some(CsvHeader::ObjectProperty {
                    parent: header.map(Box::new),
                    key: key.to_string(),
                })
            };

            if s[s_index..].starts_with('.') {
                s_index += 1;
            } else if !s[s_index..].starts_with('[') && !s[s_index..].is_empty() {
                return Err(anyhow!("Invalid CSV header '{}' - expected '.' or '['.", s));
            }
        }

        header.ok_or_else(|| anyhow!("Values in header row cannot be empty."))
    }

    pub fn components_from_parent_to_child(&self) -> Vec<&CsvHeader> {
        let mut components = Vec::new();

        let mut current = self;

        while !matches!(
            current,
            CsvHeader::ObjectProperty { parent: None, .. }
                | CsvHeader::ArrayElement { parent: None, .. }
        ) {
            components.insert(0, current);
            match current {
                CsvHeader::ArrayElement { parent, .. }
                | CsvHeader::ObjectProperty { parent, .. } => current = parent.as_ref().unwrap(),
            }
        }
        components.insert(0, current);

        components
    }
}

impl fmt::Display for CsvHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ArrayElement {
                parent: Some(parent),
                index,
                ..
            } => write!(f, "{}[{}]", parent, index),
            Self::ArrayElement {
                parent: None,
                index,
                ..
            } => write!(f, "[{}]", index),
            Self::ObjectProperty {
                parent: Some(parent),
                key,
            } => write!(f, "{}.{}", parent, key),
            Self::ObjectProperty { parent: None, key } => write!(f, "{}", key),
        }
    }
}

/// Recursively parses nested `Content` into a set of CSV headers.
fn parse_content_to_headers(
    parent: CsvHeader,
    content: &Content,
    namespace: &Namespace,
) -> Result<Vec<CsvHeader>> {
    match content {
        Content::Object(obj) => parse_object_to_headers(Some(&parent), obj, namespace),
        Content::Array(array) => parse_array_to_headers(Some(&parent), array, namespace),
        Content::OneOf(_) => parse_one_of_to_headers(parent, content, namespace),
        Content::SameAs(same_as) => parse_same_as_to_headers(parent, same_as, namespace),
        Content::Unique(unique) => parse_content_to_headers(parent, &unique.content, namespace),
        _ => Ok(vec![parent]),
    }
}

fn parse_object_to_headers(
    parent: Option<&CsvHeader>,
    obj: &ObjectContent,
    ns: &Namespace,
) -> Result<Vec<CsvHeader>> {
    let mut flatterned = Vec::new();

    for (field_name, field_content) in &obj.fields {
        flatterned.extend(parse_content_to_headers(
            CsvHeader::ObjectProperty {
                parent: parent.cloned().map(Box::new),
                key: field_name.clone(),
            },
            field_content,
            ns,
        )?);
    }

    Ok(flatterned)
}

fn parse_array_to_headers(
    parent: Option<&CsvHeader>,
    array: &ArrayContent,
    ns: &Namespace,
) -> Result<Vec<CsvHeader>> {
    let max_length = determine_content_array_max_length(array);

    let mut headers = Vec::new();

    for index in 0..max_length {
        headers.extend(
            parse_content_to_headers(
                CsvHeader::ArrayElement {
                    parent: parent.cloned().map(Box::new),
                    index,
                    max_length,
                },
                &array.content,
                ns,
            )?
            .into_iter(),
        );
    }

    Ok(headers)
}

fn parse_same_as_to_headers(
    parent: CsvHeader,
    same_as: &SameAsContent,
    ns: &Namespace,
) -> Result<Vec<CsvHeader>> {
    // Should be safe to unwrap as references have already been checked.
    let same_as_node = ns.get_s_node(&same_as.ref_).unwrap();
    parse_content_to_headers(parent, same_as_node, ns)
}

fn parse_one_of_to_headers(
    parent: CsvHeader,
    content: &Content,
    ns: &Namespace,
) -> Result<Vec<CsvHeader>> {
    if !content.is_scalar(ns)? {
        return Err(anyhow::anyhow!(
            "All variants in a 'one_of' generator must be scalar when exporting to CSV"
        ));
    }
    Ok(vec![parent])
}

#[cfg(test)]
mod tests {
    use super::*;
    use synth_core::schema::{
        number_content, BoolContent, FieldRef, NullContent, NumberContent, OneOfContent, RangeStep,
        SameAsContent, VariantContent,
    };

    use std::collections::BTreeMap;

    fn assert_csv_header_str_conversion(s: &str) {
        assert_eq!(&CsvHeader::from_csv_str(s).unwrap().to_string(), s);
    }

    #[test]
    fn test_csv_header_from_csv_str() {
        for s in &["abc", "abc.def", "a.b.c", "a[0]", "[1][2][3]", "a[12].x"] {
            assert_csv_header_str_conversion(s);
        }

        for s in &["", "a..b", "a[1]x"] {
            assert!(CsvHeader::from_csv_str(s).is_err());
        }
    }

    #[test]
    fn test_components_from_parent_to_child() {
        let root = CsvHeader::ObjectProperty {
            key: "root".to_string(),
            parent: None,
        };
        let middle = CsvHeader::ObjectProperty {
            parent: Some(Box::new(root.clone())),
            key: "x".to_string(),
        };
        let child = CsvHeader::ArrayElement {
            parent: Some(Box::new(middle.clone())),
            index: 0,
            max_length: 1,
        };

        assert_eq!(
            child.components_from_parent_to_child(),
            vec![&root, &middle, &child],
        );
    }

    #[test]
    fn test_content_to_csv_header_record() {
        let content = Content::Object(ObjectContent {
            fields: {
                let mut m = BTreeMap::new();

                m.insert(
                    "w".to_string(),
                    Content::Object(ObjectContent {
                        fields: {
                            let mut w = BTreeMap::new();
                            w.insert(
                                "a".to_string(),
                                Content::Object(ObjectContent {
                                    fields: {
                                        let mut a = BTreeMap::new();
                                        a.insert(
                                            "b".to_string(),
                                            Content::Bool(BoolContent::Constant(false)),
                                        );
                                        a
                                    },
                                    ..Default::default()
                                }),
                            );
                            w
                        },
                        ..Default::default()
                    }),
                );

                m.insert(
                    "x".to_string(),
                    Content::Array(ArrayContent {
                        length: Box::new(Content::Number(NumberContent::U64(
                            number_content::U64::Constant(2),
                        ))),
                        content: Box::new(Content::Array(ArrayContent {
                            length: Box::new(Content::Number(NumberContent::U64(
                                number_content::U64::Range(RangeStep {
                                    low: Some(1),
                                    high: Some(4),
                                    step: Some(1),
                                    ..Default::default()
                                }),
                            ))),
                            content: Box::new(Content::Null(NullContent)),
                        })),
                    }),
                );

                m.insert(
                    "y".to_string(),
                    Content::OneOf(OneOfContent {
                        variants: vec![
                            VariantContent::new(Content::Null(NullContent)),
                            VariantContent::new(Content::SameAs(SameAsContent {
                                ref_: FieldRef::new("my_collection.z").unwrap(),
                            })),
                        ],
                    }),
                );

                m.insert("z".to_string(), Content::Bool(BoolContent::Constant(true)));

                m
            },
            ..Default::default()
        });

        let mut namespace = Namespace::new();
        namespace
            .put_collection("my_collection".to_string(), content.clone())
            .unwrap();

        assert_eq!(
            CsvHeaders::from_content(&content, &namespace)
                .unwrap()
                .to_csv_record(),
            csv::StringRecord::from(vec![
                "w.a.b".to_string(),
                "x[0][0]".to_string(),
                "x[0][1]".to_string(),
                "x[0][2]".to_string(),
                "x[1][0]".to_string(),
                "x[1][1]".to_string(),
                "x[1][2]".to_string(),
                "y".to_string(),
                "z".to_string(),
            ])
        );
    }
}

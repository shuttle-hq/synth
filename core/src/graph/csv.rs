use std::fmt;

use crate::schema::content::{number_content, ArrayContent, NumberContent, ObjectContent};
use crate::{Content, Value};
use synth_gen::value::Number;

pub struct CsvHeaders {
    headers: Vec<CsvHeader>,
}

impl CsvHeaders {
    pub fn new(content: &Content) -> Self {
        println!("{:?}", content);

        match content {
            Content::Array(array_content) => {
                let headers = match &*array_content.content {
                    Content::Object(obj) => {
                        let mut v = Vec::new();
                        for (name, value) in &obj.fields {
                            v.extend(parse_to_header(CsvHeader::Simple(name.clone()), value));
                        }
                        v
                    }
                    Content::Array(array) => {
                        parse_array_to_headers(&CsvHeader::Simple("array".to_string()), array)
                    }
                    Content::OneOf(_one_of) => unimplemented!(), // limit to just atomic types?
                    Content::SameAs(_same_as) => unimplemented!(),
                    Content::Unique(_unique) => unimplemented!(),
                    _ => vec![CsvHeader::Simple("value".to_string())],
                };

                CsvHeaders { headers }
            }
            _ => panic!("Outer-most `Content` of collection should be an array"),
        }
    }
}

impl fmt::Display for CsvHeaders {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.headers.is_empty() {
            for header in self.headers.iter().take(self.headers.len() - 1) {
                write!(f, "{},", header)?;
            }
            write!(f, "{}", self.headers.last().unwrap())
        } else {
            Ok(())
        }
    }
}

/// Recursively parses nested `Content` into a set of CSV headers.
fn parse_to_header(parent: CsvHeader, content: &Content) -> Vec<CsvHeader> {
    match content {
        Content::Object(obj) => parse_object_to_headers(&parent, obj),
        Content::Array(array) => parse_array_to_headers(&parent, array),
        Content::OneOf(_one_of) => unimplemented!(), // limit to just atomic types?
        Content::SameAs(_same_as) => unimplemented!(),
        Content::Unique(_unique) => unimplemented!(),
        _ => vec![parent],
    }
}

fn parse_object_to_headers(parent: &CsvHeader, obj: &ObjectContent) -> Vec<CsvHeader> {
    let mut flatterned = Vec::new();

    for (field_name, field_content) in &obj.fields {
        flatterned.extend(parse_to_header(
            CsvHeader::ObjectProperty {
                parent: Box::new(parent.clone()),
                key: field_name.clone(),
            },
            field_content,
        ));
    }

    flatterned
}

fn parse_array_to_headers(parent: &CsvHeader, array: &ArrayContent) -> Vec<CsvHeader> {
    let max_length = determine_content_array_max_length(array);

    (0..max_length)
        .map(|index| CsvHeader::ArrayElement {
            parent: Box::new(parent.clone()),
            index,
        })
        .collect()
}

fn determine_content_array_max_length(array_content: &ArrayContent) -> usize {
    if let Content::Number(NumberContent::U64(num)) = &*array_content.length {
        (match num {
            number_content::U64::Constant(constant) => *constant,
            number_content::U64::Range(step) => {
                let high = step.high.unwrap_or(u64::MAX);
                if step.include_high {
                    high
                } else {
                    high - 1
                }
            }
            _ => panic!("Array's length should either be a constant or a range"),
        }) as usize
    } else {
        panic!("Array's length should be a number generator")
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CsvHeader {
    Simple(String),
    ArrayElement {
        parent: Box<CsvHeader>,
        index: usize,
    },
    ObjectProperty {
        parent: Box<CsvHeader>,
        key: String,
    },
}

impl fmt::Display for CsvHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Simple(x) => write!(f, "{}", x),
            Self::ArrayElement { parent, index } => write!(f, "{}[{}]", parent, index),
            Self::ObjectProperty { parent, key } => write!(f, "{}.{}", parent, key),
        }
    }
}

pub fn synth_val_to_csv(val: Value) -> String {
    match val {
        Value::Null(()) => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => synth_num_to_csv(n),
        Value::String(s) => s,
        Value::DateTime(dt) => dt.format_to_string(),
        Value::Object(_obj_map) => {
            panic!()
        }
        Value::Array(_array_vals) => {
            panic!()
        }
    }
}

fn synth_num_to_csv(n: Number) -> String {
    match n {
        Number::I8(i8) => i8.to_string() + "i8",
        Number::I16(i16) => i16.to_string() + "i16",
        Number::I32(i32) => i32.to_string() + "i32",
        Number::I64(i64) => i64.to_string(),
        Number::I128(i128) => i128.to_string() + "i128",
        Number::U8(u8) => u8.to_string() + "u8",
        Number::U16(u16) => u16.to_string() + "u16",
        Number::U32(u32) => u32.to_string() + "u32",
        Number::U64(u64) => u64.to_string() + "u64",
        Number::U128(u128) => u128.to_string() + "u128",
        Number::F32(f32) => f32.to_string() + "f32",
        Number::F64(f64) => f64.to_string() + "f64",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::schema::content::{BoolContent, NullContent, ObjectContent};

    #[test]
    fn test_parse_to_header() {
        let content = Content::Object(ObjectContent {
            skip_when_null: false,
            fields: {
                let mut map = BTreeMap::new();
                map.insert(
                    "nested".to_string(),
                    Content::Object(ObjectContent {
                        skip_when_null: false,
                        fields: {
                            let mut contained = BTreeMap::new();
                            contained.insert(
                                "field1".to_string(),
                                Content::Bool(BoolContent::Constant(false)),
                            );
                            contained.insert(
                                "array".to_string(),
                                Content::Array(ArrayContent {
                                    length: Box::new(Content::Number(NumberContent::U64(
                                        number_content::U64::Constant(2),
                                    ))),
                                    content: Box::new(Content::Null(NullContent)),
                                }),
                            );
                            contained
                        },
                    }),
                );
                map
            },
        });

        let parent = Box::new(CsvHeader::ObjectProperty {
            key: "nested".to_string(),
            parent: Box::new(CsvHeader::Simple("root".to_string())),
        });

        let array_parent = Box::new(CsvHeader::ObjectProperty {
            key: "array".to_string(),
            parent: parent.clone(),
        });

        let parsed = parse_to_header(CsvHeader::Simple("root".to_string()), &content);

        assert_eq!(
            parsed,
            vec![
                CsvHeader::ArrayElement {
                    parent: array_parent.clone(),
                    index: 0
                },
                CsvHeader::ArrayElement {
                    parent: array_parent,
                    index: 1
                },
                CsvHeader::ObjectProperty {
                    key: "field1".to_string(),
                    parent
                },
            ]
        );

        assert_eq!(
            CsvHeaders { headers: parsed }.to_string(),
            "root.nested.array[0],root.nested.array[1],root.nested.field1"
        );
    }
}

mod headers;

use crate::cli::export::{ExportParams, ExportStrategy};
use crate::sampler::{Sampler, SamplerOutput};

use synth_core::schema::content::{number_content, ArrayContent, NumberContent};
use synth_core::schema::{MergeStrategy, OptionalMergeStrategy};
use synth_core::{Content, Value};
use synth_gen::value::Number;

use anyhow::Result;

use std::convert::TryFrom;
use std::path::PathBuf;

use super::import::ImportStrategy;

#[derive(Clone, Debug)]
pub struct CsvFileExportStrategy {
    pub to_dir: PathBuf,
}

impl ExportStrategy for CsvFileExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        if self.to_dir.exists() {
            return Err(anyhow::anyhow!("Output directory already exists"));
        } else {
            std::fs::create_dir_all(&self.to_dir)?;
        }

        match csv_output_from_sampler_ouput(output.clone(), &params.namespace)? {
            CsvOutput::Namespace(ns) => {
                for (name, csv) in ns {
                    std::fs::write(self.to_dir.join(name + ".csv"), csv)?;
                }
            }
            CsvOutput::SingleCollection(csv) => {
                std::fs::write(self.to_dir.join("collection.csv"), csv)?;
            }
        }

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct CsvStdoutExportStrategy;

impl ExportStrategy for CsvStdoutExportStrategy {
    fn export(&self, params: ExportParams) -> Result<SamplerOutput> {
        let generator = Sampler::try_from(&params.namespace)?;
        let output = generator.sample_seeded(params.collection_name, params.target, params.seed)?;

        match csv_output_from_sampler_ouput(output.clone(), &params.namespace)? {
            CsvOutput::Namespace(ns) => {
                for (name, csv) in ns {
                    println!("\n{}\n{}\n\n{}\n", name, "-".repeat(name.len()), csv)
                }
            }
            CsvOutput::SingleCollection(csv) => println!("{}", csv),
        }

        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct CsvFileImportStrategy {
    pub from_dir: PathBuf,
    pub expect_header_row: bool,
}

impl ImportStrategy for CsvFileImportStrategy {
    fn import_namespace(&self) -> Result<Content> {
        let mut namespace = Content::new_object();

        for entry in std::fs::read_dir(&self.from_dir)? {
            let entry = entry?;

            // Should a non-file in the directory be an error? Or should we just silently ignore?
            if entry.file_type()?.is_file() {
                let reader = csv::ReaderBuilder::new()
                    .has_headers(self.expect_header_row)
                    .from_path(entry.path())?;

                let collection = import_csv_collection(reader, self.expect_header_row)?;

                let mut name_string = entry.file_name().into_string().map_err(|_| {
                    anyhow!("Failed to interpret collection name when importing a CSV namespace")
                })?;
                if name_string.ends_with(".csv") {
                    name_string.truncate(name_string.len() - 4);
                }

                namespace.put_collection(name_string, collection)?;
            }
        }

        Ok(namespace)
    }
}

#[derive(Clone, Debug)]
pub struct CsvStdinImportStrategy {
    pub expect_header_row: bool,
}

impl ImportStrategy for CsvStdinImportStrategy {
    fn import_namespace(&self) -> Result<Content> {
        let stdin = std::io::stdin();
        let reader = csv::ReaderBuilder::new()
            .has_headers(self.expect_header_row)
            .from_reader(stdin.lock());

        let name = "collection".to_string();
        import_csv_collection(reader, self.expect_header_row).map(|collection| {
            let mut namespace = Content::new_object();
            namespace.put_collection(name, collection).unwrap();
            namespace
        })
    }
}

pub fn import_csv_collection(
    mut reader: csv::Reader<impl std::io::Read>,
    expect_header_row: bool,
) -> Result<Content> {
    let headers = if expect_header_row {
        Some(headers::CsvHeaders::from_csv_header_record(
            &reader.headers()?.clone(),
        )?)
    } else {
        None
    };

    let mut records = reader.records();

    let head = csv_record_to_value(
        &records
            .next()
            .unwrap_or_else(|| Ok(csv::StringRecord::new()))?,
        &headers,
    )?;
    let tail = records
        .map(|res| res.map(|record| csv_record_to_value(&record, &headers)))
        .collect::<csv::Result<Result<Vec<serde_json::Value>>>>()??;

    let mut content = Content::new_collection((&head).into());

    let mut values = vec![head];
    values.extend(tail.into_iter());

    OptionalMergeStrategy.try_merge(&mut content, &serde_json::Value::Array(values))?;

    Ok(content)
}

fn csv_record_to_value(
    row: &csv::StringRecord,
    headers_opt: &Option<headers::CsvHeaders>,
) -> Result<serde_json::Value> {
    if let Some(headers) = headers_opt {
        let first_header = match headers.iter().next() {
            Some(h) => h,
            None => return Ok(serde_json::Value::Null),
        };

        let mut final_value = match first_header.components_from_parent_to_child().first() {
            Some(headers::CsvHeader::ArrayElement { .. }) => serde_json::Value::Array(Vec::new()),
            Some(headers::CsvHeader::ObjectProperty { .. }) => {
                serde_json::Value::Object(serde_json::Map::new())
            }
            None => return Ok(serde_json::Value::Null),
        };

        for (header, s) in headers.iter().zip(row.iter()) {
            let mut current_value = &mut final_value;

            let mut components = header
                .components_from_parent_to_child()
                .into_iter()
                .peekable();

            while let Some(component) = components.next() {
                let to_insert = match components.peek() {
                    Some(headers::CsvHeader::ArrayElement { .. }) => {
                        serde_json::Value::Array(Vec::new())
                    }
                    Some(headers::CsvHeader::ObjectProperty { .. }) => {
                        serde_json::Value::Object(serde_json::Map::new())
                    }
                    None => csv_str_to_value(s),
                };

                current_value = match component {
                    headers::CsvHeader::ArrayElement { index, .. } => {
                        let current_as_array = current_value.as_array_mut().unwrap();

                        if current_as_array.len() == *index {
                            current_as_array.push(to_insert);
                        } else if current_as_array.is_empty()
                            || current_as_array.len() - 1 != *index
                        {
                            return Err(anyhow!(
                                "Invalid CSV headers - array indices should increase incrementally from 0."
                            ));
                        }

                        current_as_array.get_mut(*index).unwrap()
                    }
                    headers::CsvHeader::ObjectProperty { key, .. } => {
                        let current_as_object = current_value.as_object_mut().unwrap();

                        current_as_object.entry(key).or_insert(to_insert);

                        current_as_object.get_mut(key).unwrap()
                    }
                }
            }
        }

        Ok(final_value)
    } else {
        let elements = row
            .iter()
            .map(csv_str_to_value)
            .enumerate()
            .map(|(i, val)| (format!("field{}", i), val))
            .collect();

        // Without headers, we can only assume the data was just a flat object.
        Ok(serde_json::Value::Object(elements))
    }
}

fn csv_str_to_value(s: &str) -> serde_json::Value {
    if s.is_empty() {
        serde_json::Value::Null
    } else if s == "true" {
        serde_json::Value::Bool(true)
    } else if s == "false" {
        serde_json::Value::Bool(false)
    } else if let Ok(unsigned) = s.trim().parse::<u64>() {
        serde_json::Value::Number(serde_json::Number::from(unsigned))
    } else if let Ok(signed) = s.trim().parse::<i64>() {
        serde_json::Value::Number(serde_json::Number::from(signed))
    } else if let Ok(float) = s.trim().parse::<f64>() {
        serde_json::Value::Number(serde_json::Number::from_f64(float).unwrap())
    } else {
        serde_json::Value::String(s.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum CsvOutput {
    Namespace(Vec<(String, String)>),
    SingleCollection(String),
}

fn csv_output_from_sampler_ouput(output: SamplerOutput, namespace: &Content) -> Result<CsvOutput> {
    Ok(match output {
        SamplerOutput::Namespace(key_values) => CsvOutput::Namespace(
            key_values
                .into_iter()
                .map(|(collection_name, values)| {
                    Ok((
                        collection_name.clone(),
                        to_csv_string(collection_name, values, namespace)?,
                    ))
                })
                .collect::<Result<Vec<(String, String)>>>()?,
        ),
        SamplerOutput::Collection(collection_name, values) => {
            CsvOutput::SingleCollection(to_csv_string(collection_name, values, namespace)?)
        }
    })
}

fn to_csv_string(
    collection_name: String,
    values: Vec<Value>,
    namespace: &Content,
) -> Result<String> {
    match namespace.get_collection(&collection_name)? {
        Content::Array(array_content) => {
            let content: &Content = &array_content.content;

            let mut writer = csv::Writer::from_writer(vec![]);

            writer.write_record(
                &headers::CsvHeaders::from_content(content, namespace)?.to_csv_record(),
            )?;

            for val in values {
                let record = synth_val_to_csv_record(val, content, namespace);
                writer.write_record(record)?;
            }

            Ok(String::from_utf8(writer.into_inner()?)?)
        }
        _ => panic!("Outer-most `Content` of collection should be an array"),
    }
}

fn synth_val_to_csv_record(val: Value, content: &Content, namespace: &Content) -> Vec<String> {
    match val {
        Value::Null(_) => vec![String::new()],
        Value::Bool(b) => vec![b.to_string()],
        Value::Number(n) => {
            vec![match n {
                Number::F32(f) => f.to_string(),
                Number::F64(f) => f.to_string(),
                _ => n.to_string(),
            }]
        }
        Value::String(s) => vec![s],
        Value::DateTime(dt) => vec![dt.format_to_string()],
        Value::Object(obj_map) => {
            let mut flatterned = Vec::new();

            match content {
                Content::Object(obj_content) => {
                    for (field, obj_val) in obj_map.into_iter() {
                        let inner_content = obj_content.fields.get(&field).unwrap();

                        flatterned.extend(
                            synth_val_to_csv_record(obj_val, inner_content, namespace).into_iter(),
                        );
                    }

                    flatterned
                }
                _ => panic!("Schema and generated data don't align"),
            }
        }
        Value::Array(elements) => {
            let mut flatterned = Vec::new();

            match content {
                Content::Array(array_content) => {
                    let expected_scalar_count = count_scalars_in_content(content, namespace);
                    let scalar_count = elements.len()
                        * count_scalars_in_content(&array_content.content, namespace);

                    let null_padding_iter = std::iter::repeat(Value::Null(()))
                        .take(expected_scalar_count - scalar_count);

                    let iter = elements.into_iter().chain(null_padding_iter).map(|elem| {
                        synth_val_to_csv_record(elem, &array_content.content, namespace)
                    });

                    for itm in iter {
                        flatterned.extend(itm.into_iter());
                    }

                    flatterned
                }
                _ => panic!("Schema and generated data don't align"),
            }
        }
    }
}

fn determine_content_array_max_length(array_content: &ArrayContent) -> usize {
    let length: &Content = &array_content.length;

    if let Content::Number(NumberContent::U64(num)) = length {
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

fn count_scalars_in_content(content: &Content, ns: &Content) -> usize {
    match content {
        Content::Array(array_content) => {
            determine_content_array_max_length(array_content)
                * count_scalars_in_content(&array_content.content, ns)
        }
        Content::Object(obj_content) => obj_content
            .iter()
            .map(|(_, x)| count_scalars_in_content(x, ns))
            .sum(),
        Content::SameAs(same_as) => {
            count_scalars_in_content(ns.get_s_node(&same_as.ref_).unwrap(), ns)
        }
        Content::OneOf(one_of) => one_of
            .variants
            .iter()
            .map(|x| count_scalars_in_content(&x.content, ns))
            .sum(),
        Content::Unique(unique) => count_scalars_in_content(&unique.content, ns),
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_csv_record_to_value() {
        assert_eq!(
            csv_record_to_value(
                &csv::StringRecord::from(vec!["true", "false", "true", "false"]),
                &Some(
                    headers::CsvHeaders::from_csv_header_record(&csv::StringRecord::from(vec![
                        "a[0][0]", "a[0][1]", "a[1][0]", "a[1][1]"
                    ]))
                    .unwrap()
                )
            )
            .unwrap(),
            serde_json::json!({
                "a": [
                    [true, false],
                    [true, false]
                ]
            })
        );

        assert!(csv_record_to_value(
            &csv::StringRecord::from(vec!["1", "2", "3"]),
            &Some(
                headers::CsvHeaders::from_csv_header_record(&csv::StringRecord::from(vec![
                    "a[0][0]", "a[0][1]", "a[2][0]", "a[1][1]"
                ]))
                .unwrap()
            )
        )
        .is_err());

        assert!(csv_record_to_value(
            &csv::StringRecord::from(vec!["1", "2"]),
            &Some(
                headers::CsvHeaders::from_csv_header_record(&csv::StringRecord::from(vec![
                    "a[1]", "a[0]"
                ]))
                .unwrap()
            )
        )
        .is_err());
    }

    #[test]
    fn test_csv_output_from_sampler_output() {
        let content = serde_json::from_str(
            "{
                \"type\": \"array\",
                \"length\": {
                    \"type\": \"number\",
                    \"subtype\": \"u64\",
                    \"range\": {
                        \"low\": 1,
                        \"high\": 2,
                        \"step\": 1
                    }
                },
                \"content\": {
                    \"type\": \"object\",
                    \"a\": {
                        \"type\": \"object\",
                        \"b\": {
                            \"type\": \"string\",
                            \"pattern\": \"hello world\"
                        },
                        \"c\": {
                            \"type\": \"same_as\",
                            \"ref\": \"collection.content.a.b\"
                        },
                        \"d\": {
                            \"type\": \"array\",
                            \"length\": {
                                \"type\": \"number\",
                                \"subtype\": \"u64\",
                                \"range\": {
                                    \"low\": 2,
                                    \"high\": 3,
                                    \"step\": 1
                                }
                            },
                            \"content\": {
                                \"type\": \"object\",
                                \"e\": {
                                    \"type\": \"bool\",
                                    \"constant\": true
                                },
                                \"f\": {
                                    \"type\": \"bool\",
                                    \"constant\": false
                                }
                            }
                        }
                    }
                }
            }",
        )
        .unwrap();

        let collection_name = "collection".to_string();

        let mut ns = Content::new_object();
        ns.put_collection(collection_name.clone(), content).unwrap();

        let generator = Sampler::try_from(&ns).unwrap();
        let output = generator
            .sample_seeded(Some(collection_name), 1, 0)
            .unwrap();

        assert_eq!(
            csv_output_from_sampler_ouput(output, &ns).unwrap(),
            CsvOutput::SingleCollection(
                concat!(
                    "a.b,a.c,a.d[0].e,a.d[0].f,a.d[1].e,a.d[1].f\n",
                    "hello world,hello world,true,false,true,false\n"
                )
                .to_string()
            )
        );
    }

    #[test]
    fn test_csv_str_to_value() {
        assert_eq!(
            csv_str_to_value("the quick brown fox"),
            serde_json::Value::String("the quick brown fox".to_string())
        );

        assert_eq!(csv_str_to_value("true"), serde_json::Value::Bool(true));
        assert_eq!(csv_str_to_value("false"), serde_json::Value::Bool(false));
        assert!(matches!(
            csv_str_to_value("TrUe"),
            serde_json::Value::String(_)
        ));

        assert_eq!(csv_str_to_value("64"), serde_json::json!(64));
        assert_eq!(csv_str_to_value("-64"), serde_json::json!(-64));
        assert_eq!(csv_str_to_value("64.1"), serde_json::json!(64.1));
    }
}

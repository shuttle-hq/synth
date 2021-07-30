use crate::cli::export::{ExportParams, ExportStrategy};
use crate::cli::import::ImportStrategy;
use crate::sampler::Sampler;
use anyhow::Result;
use mongodb::bson::Bson;
use mongodb::options::FindOptions;
use mongodb::{bson::Document, options::ClientOptions, sync::Client};
use serde_json::Value;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::iter::FromIterator;
use std::str::FromStr;
use synth_core::graph::prelude::content::number_content::U64;
use synth_core::graph::prelude::number_content::I64;
use synth_core::graph::prelude::{NumberContent, ObjectContent, RangeStep};
use synth_core::schema::number_content::F64;
use synth_core::schema::{
    ArrayContent, BoolContent, Categorical, ChronoValueType, DateTimeContent, FieldContent,
    OneOfContent, RegexContent, StringContent,
};
use synth_core::{Content, Name, Namespace};

#[derive(Clone, Debug)]
pub struct MongoExportStrategy {
    pub uri: String,
}

#[derive(Clone, Debug)]
pub struct MongoImportStrategy {
    pub uri: String,
}

impl ImportStrategy for MongoImportStrategy {
    fn import(self) -> Result<Namespace> {
        let client_options = ClientOptions::parse(&self.uri)?;

        info!("Connecting to database at {} ...", &self.uri);

        let client = Client::with_options(client_options)?;

        let db_name = parse_db_name(&self.uri)?;

        // 0: Initialise empty Namespace
        let mut namespace = Namespace::default();
        let database = client.database(db_name);

        // 1: First pass - create master schema
        for collection_name in database.list_collection_names(None)? {
            let collection = database.collection(&collection_name);

            // This may be useful later
            // let count = collection.estimated_document_count(None)?;

            if let Ok(Some(some_obj)) = collection.find_one(None, None) {
                let as_array = Content::Array(ArrayContent::from_content_default_length(
                    doc_to_content(&some_obj),
                ));
                namespace.put_collection(&Name::from_str(&collection_name)?, as_array)?;
            } else {
                info!("Collection {} is empty. Skipping...", collection_name);
                continue;
            }
        }

        // 2: Run an ingest step with 10 documents
        for collection_name in database.list_collection_names(None)? {
            let collection = database.collection(&collection_name);

            // This may be useful later
            // let count = collection.estimated_document_count(None)?;

            let mut find_options = FindOptions::default();
            find_options.limit = Some(10);

            let mut random_sample: Vec<Document> = collection
                .find(None, find_options)?
                .collect::<Result<Vec<Document>, _>>()?;

            random_sample.iter_mut().for_each(|doc| {
                doc.remove("_id");
            });

            namespace.default_try_update(
                &Name::from_str(&collection_name)?,
                &serde_json::to_value(random_sample)?,
            )?;
        }

        Ok(namespace)
    }

    fn import_collection(self, name: &Name) -> Result<Content> {
        self.import()?.collections.remove(name).ok_or_else(|| anyhow!(
            "Could not find table '{}' in MongoDb database.",
            name
        ))
    }

    fn into_value(self) -> Result<Value> {
        unreachable!()
    }
}

fn doc_to_content(doc: &Document) -> Content {
    let mut root = BTreeMap::new();

    // Notice this `filter` here is a hack as we don't support id's out of the box.
    for (name, bson) in doc.iter().filter(|(name, _)| name.as_str() != "_id") {
        let fc = FieldContent::new(bson_to_content(bson));
        root.insert(name.clone(), fc);
    }
    Content::Object(ObjectContent { fields: root })
}

fn bson_to_content(bson: &Bson) -> Content {
    match bson {
        Bson::Double(d) => Content::Number(NumberContent::F64(F64::Range(RangeStep {
            low: *d,
            high: *d + 1.0,
            step: 0.1,
        }))),
        Bson::String(_) => Content::String(StringContent::default()),
        Bson::Array(array) => {
            let length = Content::Number(NumberContent::U64(U64::Constant(array.len() as u64)));
            let content_iter = array.iter().map(|bson| bson_to_content(bson));

            Content::Array(ArrayContent {
                length: Box::new(length),
                content: Box::new(Content::OneOf(OneOfContent::from_iter(content_iter))),
            })
        }
        Bson::Document(doc) => doc_to_content(doc),
        Bson::Boolean(_) => Content::Bool(BoolContent::Categorical(Categorical::default())),
        Bson::Null => Content::Null,
        Bson::RegularExpression(regex) => Content::String(StringContent::Pattern(
            RegexContent::pattern(regex.pattern.clone()).unwrap_or_default(),
        )),
        Bson::JavaScriptCode(_) => {
            Content::String(StringContent::Categorical(Categorical::default()))
        }
        Bson::JavaScriptCodeWithScope(_) => {
            Content::String(StringContent::Categorical(Categorical::default()))
        }
        Bson::Int32(i) => Content::Number(NumberContent::I64(I64::Range(RangeStep {
            low: *i as i64,
            high: *i as i64 + 1,
            step: 1,
        }))),
        Bson::Int64(i) => Content::Number(NumberContent::I64(I64::Range(RangeStep {
            low: *i,
            high: *i + 1,
            step: 1,
        }))),
        Bson::DateTime(_) => Content::String(StringContent::DateTime(DateTimeContent {
            format: "".to_string(),
            type_: ChronoValueType::DateTime,
            begin: None,
            end: None,
        })),
        // There should be a more explicit enumeration here, but we don't support
        // all the required types here.
        _ => Content::String(StringContent::default()),
    }
}

impl ExportStrategy for MongoExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let mut client = Client::with_uri_str(&self.uri)?;
        let sampler = Sampler::try_from(&params.namespace)?;
        let values =
            sampler.sample_seeded(params.collection_name.clone(), params.target, params.seed)?;

        match values {
            Value::Array(collection_json) => {
                self.insert_data(params.collection_name.unwrap().to_string(), &collection_json, &mut client)
            }
            Value::Object(namespace_json) => {
                for (collection_name, collection_json) in namespace_json {
                    self.insert_data(
                        collection_name,
                        collection_json
                            .as_array()
                            .expect("This is always a collection (sampler contract)"),
                        &mut client,
                    )?;
                }
                Ok(())
            }
            _ => unreachable!(
                "The sampler will never generate a value which is not an array or object (sampler contract)"
            ),
        }
    }
}

impl MongoExportStrategy {
    fn insert_data(
        &self,
        collection_name: String,
        collection: &[Value],
        client: &mut Client,
    ) -> Result<()> {
        let db_name = parse_db_name(&self.uri)?;

        let mut docs = Vec::new();

        // Here we're going directly from JSON -> BSON.
        // This is a good first step, however there is bson type information which is lost.
        // For example, using this method we'll never get types like Bson::DateTime.
        for value in collection {
            docs.push(match value.clone().try_into()? {
                Bson::Document(doc) => doc,
                _ => bail!("invalid json document"),
            });
        }

        let n_values = docs.len();

        client
            .database(db_name)
            .collection(&collection_name)
            .insert_many(docs, None)?;

        info!(
            "Inserted {} rows into collection {} ...",
            n_values, collection_name
        );

        Ok(())
    }
}

fn parse_db_name(uri: &str) -> Result<&str> {
    // this may require a parser instead of `split`
    uri.split('/').last().ok_or_else(|| anyhow!(
        "Cannot export data. No database name specified in the uri"
    ))
}

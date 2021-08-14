use synth_core::{Content, Name, Namespace, schema::{ArrayContent, ObjectContent}};
use std::{convert::TryFrom, fs::{File, OpenOptions}, io::{BufWriter, Write}, path::{Path, PathBuf}, str::FromStr};
use crate::sampler::Sampler;
use super::{ExportStrategy, ExportParams};
use anyhow::{Result, Context};

#[derive(Clone, Debug)]
pub struct DocExportStrategy{
    file_path: PathBuf
}

impl ExportStrategy for DocExportStrategy{
    fn export(self, params: ExportParams) -> Result<()> {
        let generator = Sampler::try_from(&params.namespace)?;
        let values = generator.sample_seeded(params.collection_name, params.target, params.seed)?;
        let file = OpenOptions::new().create(true).read(true).append(true).open(self.file_path.clone())?;
        let mut file = BufWriter::new(file);
        write_doc(&params.namespace, &mut file)?;
        //example result
        writeln!(&mut file, "```json")?;
        serde_json::to_writer_pretty(&mut file, &values)?;
        writeln!(file, "")?;
        writeln!(&mut file, "```")?;
        writeln!(file, "> Created with [synth](https://github.com/getsynth/synth)")?;
        Ok(())
    }
}

impl DocExportStrategy {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file_path = Path::join(&path, Path::new("README.md"));
        Ok(
            Self {
                file_path
            }
        )
    }
}

fn write_doc(ns: &Namespace, file: &mut BufWriter<File>) -> Result<()>{ 
        write_colls(ns, file)?; 
        writeln!(file, "---")?;
        let pairs = ns.iter();
        for (n,c) in pairs {
            write_coll_details((n,c), file)?;
            writeln!(file, "---")?;
        };
        writeln!(file, "---")?;
        writeln!(file, "### Example")?;
        Ok(())
    }

fn write_colls(ns: &Namespace, file: &mut BufWriter<File>) -> Result<()> {
    writeln!(file, "```text\n### Description\n\n\n\n\n```")?;
    writeln!(file, "## Collections")?;
    for key in ns.keys().map(|n| n.as_ref()) {
        writeln!(file, "- [{}]({})", key, format!("#collection-{}", key)).context("error while writing to file")?;
    };
    Ok(())
}
fn write_coll_details((name, content ): (&Name,&Content), file: &mut BufWriter<File>) -> Result<()>{
    match content {
        Content::Array(ArrayContent { content: box Content::Object(object_content), .. }) => {
            let name_str = format!("Collection {}", name);
            writeln!(file, "#### {}", name_str)?;
            let name = Name::from_str("Collection")?;
            write_coll_details((&name, &Content::Object(object_content.clone()) ), file)?;
            write_foreigns((&name,  &object_content), file)?;
        }
        Content::Object(obj) => {
            writeln!(file, "##### Fields")?;
            let ObjectContent { fields } = obj;
            for (field_name, field_content) in fields.into_iter() {
                let cont = &field_content.content;
                let f_name = &Name::from_str(field_name)?;
                if let box Content::Array(arr) = &cont {
                    write_subcoll(name, (f_name, arr), file)?;
                } else {
                    write_coll_details((f_name, cont ), file)?;
                }
            } 
        },
        Content::Null => {
            writeln!(file, "##### {} [Type: *Null*]", name)?;
            writeln!(file, "> Description goes here: ")?;
            writeln!(file, "")?;
            writeln!(file, "")?;
        }
        Content::Bool(_) | Content::String(_) | Content::Number(_) | Content::SameAs(_)=> {
            writeln!(file, "##### {} [Type: *{}*]", name, content.kind())?;
            writeln!(file, "> Description goes here: ")?;
            writeln!(file, "")?;
            writeln!(file, "")?;
        }
        Content::OneOf(one_of) => {
            writeln!(file, "##### {} [Type: *{}*]", name, content.kind())?;
            writeln!(file, "> Description goes here: ")?;
            writeln!(file, "**Variants:**")?;
            for one in one_of.iter() {
                write_coll_details((name, one ), file)?;
            };
        }
        _ => {

        }
    };
    Ok(())
}

fn write_foreigns((name, obj):(&Name, &ObjectContent), file: &mut BufWriter<File>) -> Result<()> {
    writeln!(file, "#### Foreign Keys")?;
    let ObjectContent { fields } = obj;
    for (_, field_content) in fields.into_iter() {
        if let box Content::SameAs(same) = &field_content.content {
            let foreign_coll = same.ref_.collection().as_ref();
            let foreign_feild = same.ref_.iter_fields().last().ok_or_else(|| anyhow::anyhow!("Could not get the last field"))?;
            writeln!(file, "- {} refers to: **field** *{}* in **Collection** [{}](#collection-{}):", name, foreign_feild, foreign_coll, foreign_coll)?;
        }
    };
    Ok(())
}

fn write_subcoll(parent: &Name, (name, arr):(&Name, &ArrayContent), file: &mut BufWriter<File>)  -> Result<()> {
    let name_str = format!("Subcollection {}: Parent: {}", name, parent);
    writeln!(file, "{}", name_str)?;
    write_coll_details((name, &Content::Array(arr.clone()) ), file)?;
    Ok(())
}
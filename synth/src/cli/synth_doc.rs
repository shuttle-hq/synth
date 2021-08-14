use super::{ExportParams, ExportStrategy};
use crate::sampler::Sampler;
use anyhow::{Context, Result};
use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use synth_core::{
    schema::{ArrayContent, ObjectContent},
    Content, Name, Namespace,
};

#[derive(Clone, Debug)]
pub struct DocExportStrategy {
    file_path: PathBuf,
}

impl ExportStrategy for DocExportStrategy {
    fn export(self, params: ExportParams) -> Result<()> {
        let generator = Sampler::new(&params.namespace);
        let values = generator.sample_seeded(params.collection_name, params.target, params.seed)?;
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(self.file_path.clone())?;
        let mut file = BufWriter::new(file);
        write_doc(&params.namespace, &mut file)?;
        //example result
        writeln!(&mut file, "```json")?;
        serde_json::to_writer_pretty(&mut file, &values)?;
        writeln!(file, "")?;
        writeln!(
            &mut file,
            "```\n> Created with [synth](https://github.com/getsynth/synth)"
        )?;
        Ok(())
    }
}

impl DocExportStrategy {
    pub fn new(path: PathBuf) -> Result<Self> {
        let file_path = Path::join(&path, Path::new("README.md"));
        Ok(Self { file_path })
    }
}

fn write_doc(ns: &Namespace, file: &mut BufWriter<File>) -> Result<()> {
    write_colls(ns, file)?;
    writeln!(file, "---")?;
    let pairs = ns.iter();
    for (n, c) in pairs {
        let subs = write_coll_details((n, c), file)?;
        if !subs.is_empty() {
            for sub in subs.iter() {
                let (name, arr, parent) = sub;
                write_subcoll(name.clone(), *arr, parent, file)?;
            }
        }
        writeln!(file, "---")?;
    }
    writeln!(file, "---\n## Example")?;
    Ok(())
}

fn write_colls(ns: &Namespace, file: &mut BufWriter<File>) -> Result<()> {
    writeln!(
        file,
        "\n# Description\n\n\n\n\n> Please enter a descriptive text here\n## Collections"
    )?;
    for key in ns.keys().map(|n| n.as_ref()) {
        writeln!(file, "- [{}]({})", key, format!("#collection-{}", key))
            .context("error while writing to file")?;
    }
    Ok(())
}
fn write_coll_details<'a>(
    (name, content): (&'a Name, &'a Content),
    file: &mut BufWriter<File>,
) -> Result<Vec<(Name, &'a ArrayContent, &'a Name)>> {
    let mut sub_colls = Vec::new();
    match content {
        Content::Array(ArrayContent {
            content: box Content::Object(object_content),
            ..
        }) => {
            let name_str = format!("Collection **{}**", name);
            writeln!(file, "## {}", name_str)?;
            let name = Name::from_str("Collection")?;
            write_coll_details((&name, &Content::Object(object_content.clone())), file)?;
            write_foreigns(&object_content, file)?;
        }
        Content::Object(obj) => {
            writeln!(file, "### Fields\n| Name   |Type  |Description( Please replace the generated texts ) |\n|--------|---------|--------|")?;
            let ObjectContent { fields } = obj;
            for (field_name, field_content) in fields.into_iter() {
                let cont = &field_content.content;
                let f_name = Name::from_str(field_name)?;
                if let box Content::Array(arr) = &cont {
                    sub_colls.push((f_name.clone(), arr, name));
                } else {
                    write_coll_details((&f_name, cont), file)?;
                }
            }
        }
        Content::Null => {
            writeln!(file, "| {} | Null | Description goes here: |", name)?;
        }
        Content::Bool(_) | Content::String(_) | Content::Number(_) | Content::SameAs(_) => {
            writeln!(
                file,
                "| {} | {} | Description goes here: |",
                name,
                content.kind()
            )?;
        }
        Content::OneOf(one_of) => {
            writeln!(
                file,
                "| {} | {} | Description goes here: |\n**Variants:**",
                name,
                content.kind()
            )?;
            let mut s = String::new();
            for one in one_of.iter() {
                s.push_str(format!("- {}\n", one.kind()).as_ref());
            }
            writeln!(file, "{}", s)?;
        }
        _ => {}
    };
    Ok(sub_colls)
}

fn write_foreigns(obj: &ObjectContent, file: &mut BufWriter<File>) -> Result<()> {
    writeln!(file, "### Foreign Keys")?;
    let ObjectContent { fields } = obj;
    if fields.is_empty() {
        writeln!(file, "No foreign keys")?;
    } else {
        for (f_name, field_content) in fields.into_iter() {
            if let box Content::SameAs(same) = &field_content.content {
                let foreign_coll = same.ref_.collection().as_ref();
                let foreign_feild = same
                    .ref_
                    .iter_fields()
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Could not get the last field"))?;
                writeln!(
                    file,
                    "- _**{}**_ refers to: **field** *{}* in **Collection** [{}](#collection-{}):",
                    f_name, foreign_feild, foreign_coll, foreign_coll
                )?;
            }
        }
    };
    Ok(())
}

fn write_subcoll(
    name: Name,
    arr: &ArrayContent,
    parent: &Name,
    file: &mut BufWriter<File>,
) -> Result<()> {
    let name_str = format!("### Sub-Collection {} of Parent {}", name, parent);
    writeln!(file, "{}", name_str)?;
    write_coll_details((&name, &Content::Array(arr.clone())), file)?;
    Ok(())
}

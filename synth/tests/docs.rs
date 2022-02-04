use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;

// Skipping fmt is needed until this fix is released
// https://github.com/rust-lang/rustfmt/pull/5142
#[rustfmt::skip]
mod helpers;

use helpers::generate;
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;

#[async_std::test]
async fn docs() -> Result<()> {
    let tests = WalkBuilder::new("../")
        .filter_entry(is_markdown)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| !is_dir(entry));

    for test in tests {
        test_doc(test).await?;
    }

    Ok(())
}

fn is_markdown(dir_entry: &DirEntry) -> bool {
    is_dir(dir_entry) || dir_entry.path().extension() == Some(OsStr::new("md"))
}

fn is_dir(dir_entry: &DirEntry) -> bool {
    dir_entry.path().is_dir()
}

async fn test_doc(dir_entry: DirEntry) -> Result<()> {
    println!("{}", dir_entry.path().display());

    let tmp = Path::new("tmp");

    let ns = get_ns_dir(tmp, &dir_entry);
    fs::create_dir_all(&ns)?;

    let mut expects = HashSet::new();

    extract_code_blocks(dir_entry.path())?
        .into_iter()
        .filter(is_json_block)
        .try_for_each(|block| -> Result<()> {
            let expect = write_code_block(&dir_entry, block, tmp)?;

            if let Some(expect) = expect {
                expects.insert(expect);
            }

            Ok(())
        })?;

    let current = env::current_dir()?;
    env::set_current_dir(tmp)?;
    let actual = generate(get_ns(&dir_entry).unwrap()).await;
    env::set_current_dir(current)?;

    // Did we expect any errors for this document
    if expects.is_empty() {
        assert!(
            actual.is_ok(),
            "should not have error: {:?}\n",
            actual.unwrap_err()
        );
    } else {
        assert!(
            actual.is_err(),
            "should be one of the following errors: {:#?}\n",
            expects
        );

        let err = actual.unwrap_err();
        let err = format!("{:?}", err);

        assert!(
            expects.iter().any(|expect| err.contains(expect)),
            "{}\nshould contain one of the following errors: {:#?}",
            err,
            expects
        );
    }

    fs::remove_dir_all(tmp)?;

    Ok(())
}

struct Line {
    content: String,
    index: usize,
}

fn extract_code_blocks(path: &Path) -> Result<Vec<Vec<Line>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut blocks = Vec::new();
    let mut block: Option<Vec<Line>> = None;
    let mut count = 0;

    for line in reader.lines() {
        let line = line?;
        count += 1;

        // We are currently in a code block
        if let Some(block) = block.as_mut() {
            block.push(Line {
                content: line.clone(),
                index: count,
            });
        }

        if line.starts_with("```") {
            // We are currently in a block so this must be an end marker
            if let Some(block_some) = block {
                blocks.push(block_some);
                block = None;
            } else {
                block = Some(vec![Line {
                    content: line,
                    index: count,
                }]);
            }
        }
    }

    Ok(blocks)
}

#[allow(clippy::ptr_arg)]
fn is_json_block(block: &Vec<Line>) -> bool {
    block.len() > 1 && block[0].content.starts_with("```json")
}

fn get_ns_dir(tmp: &Path, dir_entry: &DirEntry) -> PathBuf {
    let ns = get_ns(dir_entry).unwrap();
    tmp.join(ns)
}

fn get_ns(dir_entry: &DirEntry) -> Option<&str> {
    dir_entry.path().file_stem()?.to_str()
}

lazy_static! {
    /// Regex to extract whether a code block has synth schema, a custom filename, or expected
    /// errors
    /// The following lines should be matched with their respective capture groups
    /// ```json synth                                       (synth= synth)
    /// ```json synth[custom.json]                          (synth= synth)(file=custom.json)
    /// ```json synth[expect = "error"]                     (synth= synth)(expect=error)
    /// ```json[data.json]                                  (file=data.json)
     static ref BLOCK_IDENTIFIER: Regex =Regex::new(
        r#"^```json(?P<synth> synth)?(?:\[(?P<file>.*\.json)\])?(?:\[expect\s=\s"(?P<expect>.*)"\])?$"#,
    ).unwrap();

    /// Regex to capture any comments at the end of a line
    static ref COMMENT: Regex = Regex::new("(?P<comment>//.*$)").unwrap();
}

fn write_code_block(dir_entry: &DirEntry, block: Vec<Line>, tmp: &Path) -> Result<Option<String>> {
    let ns = get_ns_dir(tmp, dir_entry);

    let (is_synth, file, expect) = BLOCK_IDENTIFIER
        .captures(&block[0].content)
        .map(|cap| {
            (
                cap.name("synth").is_some(),
                cap.name("file").map(|file| file.as_str().to_string()),
                cap.name("expect").map(|expect| expect.as_str().to_string()),
            )
        })
        .unwrap();

    let file = match (is_synth, file) {
        (false, None) => {
            println!(
                "{}:{} has a JSON only code block that will be skipped",
                dir_entry.path().display(),
                block[0].index
            );
            return Ok(None);
        }
        (false, Some(file)) => {
            println!(
                "{}:{} has a JSON data file that will be copied",
                dir_entry.path().display(),
                block[0].index
            );
            tmp.join(file)
        }
        (true, Some(file)) => ns.join(file),
        (true, None) => ns.join(format!("{}.json", block[0].index)),
    };

    let mut file = File::create(file)?;
    let is_array_block = block[2].content.contains("\"type\": \"array\",");

    if is_synth && !is_array_block {
        write!(
            file,
            r#"{{
            "type": "array",
            "length": 1,
            "content": "#
        )?;
    }

    for line in &block[1..block.len() - 1] {
        // Strip comments since they will crash serde
        let comment = COMMENT
            .captures(&line.content)
            .map(|cap| {
                cap.name("comment")
                    .map(|comment| comment.as_str().to_string())
            })
            .unwrap_or(None);

        let mut safe_line = line.content.clone();

        if let Some(comment) = comment {
            safe_line = safe_line.trim_end_matches(&comment).to_string();
        }

        writeln!(file, "{}", safe_line)?;
    }

    if is_synth && !is_array_block {
        writeln!(file, "}}")?;
    }

    Ok(expect)
}

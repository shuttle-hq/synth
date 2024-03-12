#![allow(clippy::needless_borrow, clippy::explicit_counter_loop)]
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};
use test_macros::tmpl_ignore;

use anyhow::Result;

// Skipping fmt is needed until this fix is released
// https://github.com/rust-lang/rustfmt/pull/5142
#[rustfmt::skip]
mod helpers;

use helpers::{generate, generate_scenario};
use regex::Regex;

#[tmpl_ignore("./", exclude_dir = true, filter_extension = "md")]
#[async_std::test]
async fn PATH_IDENT() -> Result<()> {
    let path = Path::new("../").join(PATH);
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tmp");
    let tmp = tmp.as_path();

    fs::create_dir_all(tmp)?;
    env::set_current_dir(tmp)?;
    let path = Path::new("../").join(path);

    let ns = get_ns_dir(tmp, &path);
    fs::create_dir_all(ns.join("scenarios"))?;

    let mut expects = HashSet::new();
    let mut scenarios = HashSet::new();

    extract_code_blocks(&path)?
        .into_iter()
        .filter(is_json_block)
        .try_for_each(|block| -> Result<()> {
            let (expect, scenario_name) = write_code_block(&path, block, tmp)?;

            if let Some(expect) = expect {
                expects.insert(expect);
            }

            if let Some(name) = scenario_name {
                scenarios.insert(name);
            }

            Ok(())
        })?;

    let ns = get_ns(&path).unwrap();
    let actual = generate(&ns).await;

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
            "should be one of the following errors: {expects:#?}\n"
        );

        let err = actual.unwrap_err();
        let err = format!("{err:?}");

        assert!(
            expects.iter().any(|expect| err.contains(expect)),
            "{err}\nshould contain one of the following errors: {expects:#?}"
        );
    }

    for scenario in scenarios {
        let actual = generate_scenario(&ns, Some(scenario.clone())).await;

        assert!(
            actual.is_ok(),
            "'{}' scenario should not have error: {:?}\n",
            scenario,
            actual.unwrap_err()
        );
    }

    fs::remove_dir_all(ns)?;

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

fn get_ns_dir(tmp: &Path, ns: &Path) -> PathBuf {
    let ns = get_ns(ns).unwrap();
    tmp.join(ns)
}

fn get_ns(path: &Path) -> Option<String> {
    let ns = path.with_extension("");

    Some(ns.strip_prefix("../../").unwrap().display().to_string())
}

lazy_static! {
    /// Regex to extract whether a code block has synth schema, a custom filename, or expected
    /// errors
    /// The following lines should be matched with their respective capture groups
    /// ```json synth                                       (synth= synth)
    /// ```json synth[custom.json]                          (synth= synth)(file=custom.json)
    /// ```json synth[expect = "error"]                     (synth= synth)(expect=error)
    /// ```json[data.json]                                  (file=data.json)
    /// ```json synth-scenario[name.json]                   (scenario= synth-scenario)(file=name.json)
     static ref BLOCK_IDENTIFIER: Regex =Regex::new(
        r#"^```json(?P<synth> synth)?(?P<scenario> synth\-scenario)?(?:\[(?P<file>.*\.json)\])?(?:\[expect\s=\s"(?P<expect>.*)"\])?$"#,
    ).unwrap();

    /// Regex to capture any comments at the end of a line
    static ref COMMENT: Regex = Regex::new("(?P<comment>//.*$)").unwrap();
}

fn write_code_block(
    ns: &Path,
    block: Vec<Line>,
    tmp: &Path,
) -> Result<(Option<String>, Option<String>)> {
    let mut ns = get_ns_dir(tmp, ns);

    let (is_synth, is_scenario, file, expect) = BLOCK_IDENTIFIER
        .captures(&block[0].content)
        .map(|cap| {
            (
                cap.name("synth").is_some(),
                cap.name("scenario").is_some(),
                cap.name("file").map(|file| file.as_str().to_string()),
                cap.name("expect").map(|expect| expect.as_str().to_string()),
            )
        })
        .unwrap();

    let scenario_name = if is_scenario {
        if let Some(ref scenario_file) = file {
            ns = ns.join("scenarios");
            Some(scenario_file.trim_end_matches(".json").to_string())
        } else {
            None
        }
    } else {
        None
    };

    let file = match ((is_synth || is_scenario), file) {
        (false, None) => {
            println!(
                "{}:{} has a JSON only code block that will be skipped",
                ns.display(),
                block[0].index
            );
            return Ok((None, None));
        }
        (false, Some(file)) => {
            println!(
                "{}:{} has a JSON data file that will be copied",
                ns.display(),
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

        // Make single line items a field in an object
        if block.len() == 3 {
            write!(
                file,
                r#"{{
            "type": "object","#
            )?;
        }
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

        writeln!(file, "{safe_line}")?;
    }

    if is_synth && !is_array_block {
        if block.len() == 3 {
            writeln!(file, "}}")?;
        }
        writeln!(file, "}}")?;
    }

    Ok((expect, scenario_name))
}

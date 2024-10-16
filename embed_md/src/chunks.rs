use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::io::Write;
use std::ops::Range;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose;
use base64::Engine;
use regex::Regex;
use sha2::{Digest, Sha256};

use embed_md_derive::RangeFn;
use embed_md_traits::FunctionTag;
use embed_md_traits::Rangeable;

use crate::helpers::extract_map;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Identity(Identity),
    ExecCode(ExecCode),
}

impl FunctionType {
    pub fn from(
        id: String,
        function: String,
        params: HashMap<String, String>,
        is_end: bool,
        range: Range<usize>,
    ) -> FunctionType {
        match function.as_str() {
            "identity" => FunctionType::Identity(Identity { id, is_end, range }),
            "exec-code" => FunctionType::ExecCode(ExecCode {
                id,
                is_end,
                params,
                range,
            }),
            _ => panic!("Not a known function",),
        }
    }
}

impl Rangeable for FunctionType {
    fn range(&self) -> Range<usize> {
        match self {
            FunctionType::Identity(i) => i.range(),
            FunctionType::ExecCode(i) => i.range(),
        }
    }

    fn id(&self) -> String {
        match self {
            FunctionType::Identity(i) => i.id(),
            FunctionType::ExecCode(i) => i.id(),
        }
    }
}

impl FunctionTag for FunctionType {
    fn transform(&self, text: String) -> String {
        match self {
            FunctionType::Identity(i) => i.transform(text),
            FunctionType::ExecCode(i) => i.transform(text),
        }
    }
}

#[derive(RangeFn, Debug, Clone, PartialEq)]
pub struct ExecCode {
    id: String,
    params: HashMap<String, String>,
    is_end: bool,
    range: Range<usize>,
}

impl FunctionTag for ExecCode {
    fn transform(&self, text: String) -> String {
        exec_code(text.as_str(), &self.params).unwrap()
    }
}

fn exec_code(text: &str, params: &HashMap<String, String>) -> Result<String, String> {
    let re = Regex::new("(```.*?\n((.*\n)*?)```)(?s)").unwrap();
    let meta_re = Regex::new(r#"<!--embed-meta +((\w*=".*?":? )*)? ?(/?)-->\n"#).unwrap();
    let result_header_re = Regex::new(r"<!-- result -->\n").unwrap();
    let meta_option = meta_re.captures(text);
    let meta = match meta_option {
        None => HashMap::new(),
        Some(t) => extract_map(&t[1]),
    };
    let (to_exec, with_block, remaining) = match re.captures(text) {
        None => {
            panic!("No code block found in text {:?}", text)
        }
        Some(t) => (
            t.get(2).unwrap().as_str(),
            t.get(1).unwrap().as_str(),
            t.get(0).unwrap().range().end..text.len(),
        ),
    };

    let result_start = match result_header_re.captures(text) {
        None => text.len() - 1,
        Some(c) => c.get(0).unwrap().range().start - 1,
    };
    let result_header = Range {
        start: remaining.start,
        end: result_start,
    };

    let default_path = "./".to_string();
    let file_loc = params.get("exec_location").unwrap_or(&default_path);
    let mut exec_loc = current_dir().unwrap();
    exec_loc.push(file_loc);
    let file_name = params.get("file_name").unwrap();
    exec_loc.push(file_name);
    let mut wrapper = Sha256::new();
    wrapper.update(
        fs::canonicalize(exec_loc)
            .unwrap()
            .to_str()
            .unwrap()
            .as_bytes(),
    );
    let output_file_hash = wrapper.finalize();
    let output_file_hash_b64 = general_purpose::URL_SAFE_NO_PAD.encode(output_file_hash.as_slice());
    let binding = "~/.embed_md".to_string();
    let out_dir_path = params.get("out_dir").unwrap_or(&binding);
    let out_dir = shellexpand::tilde(out_dir_path).to_string();
    // Check if outdir exists if it doesn't create it
    match fs::metadata(&out_dir) {
        Ok(_) => (),
        // Make this resilient to tests running in parallel in CI
        Err(_) => match fs::create_dir(&out_dir) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => panic!("Error creating directory: {}", e),
            },
        },
    }

    let id_match = Regex::new(r"\$\$(.*?)\$\$").unwrap();
    let exec_replaced = id_match.replace_all(
        to_exec,
        format!("{}/{}_$1.out", out_dir, output_file_hash_b64),
    );
    let mut wrapper = Sha256::new();
    wrapper.update(exec_replaced.as_bytes());
    let result = wrapper.finalize();
    let b64 = general_purpose::STANDARD_NO_PAD.encode(result.as_slice());

    let existing_hash = meta.get("hash").map_or("", String::as_str);
    let last_run = meta.get("last_run").map_or("0", String::as_str);
    let rerun = match params.get("cache") {
        Some(x) => match x.as_str() {
            "always" => false,
            "hash" => b64 != *existing_hash,
            "time" => {
                todo!("Need to implement time")
            }
            _ => true,
        },
        _ => true,
    };

    match rerun {
        true => {
            let lang = params.get("lang").map_or("shell", String::as_str);
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let exec_path = params.get("exec_path");
            let r_exec_path = params.get("r_exec_path");
            let path = match exec_path {
                Some(path) => path.to_string(),
                None => match r_exec_path {
                    Some(path) => format!("{}/{}", file_loc, path),
                    None => file_loc.to_string(),
                },
            };
            let exec_lang = match lang {
                "shell" | "sh" => "sh",
                "zsh" => "zsh",
                "bash" => "bash",
                s if s.starts_with("python") => "python3",
                _ => {
                    todo!("Need to implement other languages: {}", lang)
                }
            };
            let mut child = Command::new(exec_lang)
                .current_dir(shellexpand::tilde(path.as_str()).to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start command");

            {
                let stdin = child.stdin.as_mut().expect("Failed to open stdin");
                stdin
                    .write_all(exec_replaced.as_bytes())
                    .expect("Failed to write to stdin");
            }

            let output = child.wait_with_output().expect("Failed to read stdout");

            let id_out = format!(
                "{}/{}_{}.out",
                out_dir,
                output_file_hash_b64,
                params.get("exec_id").unwrap()
            );
            match fs::write(id_out.clone(), &output.stdout) {
                Ok(_) => (),
                Err(e) => panic!("Error writing to file: {}, {}", id_out, e),
            }

            let maybe_new_line = match output.stdout.ends_with(&[10]) {
                true => "",
                false => "\n",
            };
            match params.get("o_lang") {
                Some(x) if x == "none" => Ok(format!(
                    "<!--embed-meta hash=\"{}\": last_run=\"{}\" -->\n",
                    b64,
                    since_the_epoch.as_millis()
                ) + with_block
                    + &text[result_header]
                    + "\n<!-- result -->\n"),
                _ => Ok(format!(
                    "<!--embed-meta hash=\"{}\": last_run=\"{}\" -->\n",
                    b64,
                    since_the_epoch.as_millis()
                ) + with_block
                    + &text[result_header]
                    + "\n<!-- result -->\n```"
                    + params.get("o_lang").map_or("", String::as_str)
                    + "\n"
                    + std::str::from_utf8(&output.stdout).unwrap()
                    + maybe_new_line
                    + "```\n"),
            }
        }
        false => Ok(format!(
            "<!--embed-meta hash=\"{}\": last_run=\"{}\" -->\n",
            b64, last_run
        ) + with_block
            + &text[remaining]),
    }
}

#[derive(RangeFn, Debug, Clone, PartialEq)]
pub struct Identity {
    id: String,
    is_end: bool,
    range: Range<usize>,
}

impl Identity {
    pub fn new(id: String, is_end: bool, range: Range<usize>) -> Self {
        Self { id, is_end, range }
    }
}
impl FunctionTag for Identity {
    fn transform(&self, text: String) -> String {
        text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXEC_RESULT: &str = r#"<!--embed-meta hash="ZtPq82dXLdZblrj0VuVOPyjQqP9cuvsoaZV7p49AbR8": last_run="1111" -->
```shell
echo "test"; echo "another"
```
<!-- result -->
```
test
another
```
"#;

    const EXEC_RESULT_WITH_RESULT_HEADER: &str = r#"<!--embed-meta hash="ZtPq82dXLdZblrj0VuVOPyjQqP9cuvsoaZV7p49AbR8": last_run="1111" -->
```shell
echo "test"; echo "another"
```
something something
<!-- result -->
```
test
another
```
"#;

    const EXEC_RESULT_WITH_MULTILINE: &str = r#"<!--embed-meta hash="inEJSzHFpeX33einbd07paa8ka8+vbC/CvC2Ka9z9Wk": last_run="1111" -->
```shell
echo "test"
echo "another"
```
<!-- result -->
```
test
another
```
"#;

    #[test]
    fn test_exec_code() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert("exec_id".to_string(), "test_exec_code".to_string());
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"```shell
echo "test"; echo "another"
```
"#,
            &params,
        );
        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT.to_string())
    }

    #[test]
    fn test_exec_code_existing() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert("exec_id".to_string(), "test_exec_code_existing".to_string());
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"```shell
echo "test"; echo "another"
```
<!-- result -->
```
test
another
```
"#,
            &params,
        );
        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT.to_string())
    }

    #[test]
    fn test_exec_code_existing_technically_legal() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert(
            "exec_id".to_string(),
            "test_exec_code_existing_technically_legal".to_string(),
        );
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"```shell
echo "test"; echo "another"
```
<!-- result -->
"#,
            &params,
        );

        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT.to_string())
    }

    #[test]
    fn test_exec_code_header_no_result() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert(
            "exec_id".to_string(),
            "test_exec_code_header_no_result".to_string(),
        );
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"```shell
echo "test"; echo "another"
```
something something
"#,
            &params,
        );

        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT_WITH_RESULT_HEADER.to_string())
    }

    #[test]
    fn test_exec_code_existing_with_result_header() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert(
            "exec_id".to_string(),
            "test_exec_code_existing_with_result_header".to_string(),
        );
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"#
                ```shell
echo "test"; echo "another"
```
something something
<!-- result -->
```
test
another
```
"#,
            &params,
        );
        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT_WITH_RESULT_HEADER.to_string())
    }

    #[test]
    fn test_exec_code_multi_line() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert(
            "exec_id".to_string(),
            "test_exec_code_multi_line".to_string(),
        );
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"```shell
echo "test"
echo "another"
```
"#,
            &params,
        );
        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT_WITH_MULTILINE.to_string())
    }

    #[test]
    fn test_exec_code_cached_pre_result_text() {
        let mut params = HashMap::new();
        params.insert("file_name".to_string(), "Cargo.toml".to_string());
        params.insert(
            "exec_id".to_string(),
            "test_exec_code_existing_with_result_header".to_string(),
        );
        params.insert("cache".to_string(), "always".to_string());
        params.insert("out_dir".to_string(), "../test_out_dir".to_string());
        let result = exec_code(
            r#"#
                ```shell
echo "test"; echo "another"
```
something something
<!-- result -->
```
test
another
```
"#,
            &params,
        );
        assert!(result.is_ok());
        let re = Regex::new(r#"(?<ts>last_run=")(\d+)""#).unwrap();
        let rest = result.unwrap();
        let static_time = re.replace(rest.as_str(), "${ts}1111\"");
        // let static_time = result.unwrap().as_str().replace(r#"last_run=""#, "last_run=\"1697141890682\"");
        assert_eq!(static_time, EXEC_RESULT_WITH_RESULT_HEADER.to_string())
    }
}

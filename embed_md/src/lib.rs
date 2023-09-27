mod chunks;
mod helpers;

use crate::chunks::{FunctionType, Identity};
use crate::helpers::extract_map;
use embed_md_traits::FunctionTag;
use embed_md_traits::Rangeable;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct TagFunctionPair {
    start: FunctionType,
    end: FunctionType,
}

impl TagFunctionPair {
    fn internal_range(&self) -> Range<usize> {
        self.start.range().end..self.end.range().start
    }
}

#[derive(Debug, Clone)]
struct Chunk {
    pair: TagFunctionPair,
    text: String,
    opening_tag: String,
    closing_tag: String,
}

impl Chunk {
    fn transform(&self, id: Option<String>) -> Chunk {
        let text = match id {
            Some(id) if id == self.pair.start.id() => self.pair.start.transform(self.text.clone()),
            Some(_) => self.text.clone(),
            _ => self.pair.start.transform(self.text.clone()),
        };
        Chunk {
            text,
            pair: self.pair.clone(),
            opening_tag: self.opening_tag.clone(),
            closing_tag: self.closing_tag.clone(),
        }
    }

    fn print_representation(&self) -> String {
        format!("{}{}{}", self.opening_tag, self.text, self.closing_tag)
    }
}

fn new_identity_chunk(range: Range<usize>, message: &str) -> Chunk {
    let tag_pair = TagFunctionPair {
        start: FunctionType::Identity(Identity::new(
            "identity".to_string(),
            false,
            range.start..range.start,
        )),
        end: FunctionType::Identity(Identity::new(
            "identity".to_string(),
            true,
            range.end..range.end,
        )),
    };
    let internal_range = tag_pair.internal_range().clone();
    Chunk {
        pair: tag_pair,
        text: message[internal_range].to_string(),
        opening_tag: "".to_string(),
        closing_tag: "".to_string(),
    }
}

fn process_to_chunks(message: &str, file: &Path) -> Vec<Chunk> {
    let re =
        regex::Regex::new(r#"<!--embed (.*?) id="(.*?)" +((\w*=".*?":? )*)? ?(/?)-->\n"#).unwrap();
    let mut to_collection: Vec<Vec<FunctionType>> = Vec::new();
    for cap in re.captures_iter(message) {
        let id = cap[2].to_string();
        let function = cap[1].to_string();
        let mut params: HashMap<String, String> = extract_map(&cap[3]);
        params.insert(
            "exec_location".to_string(),
            file.parent().unwrap().to_str().unwrap().to_string(),
        );
        params.insert(
            "file_name".to_string(),
            file.file_name().unwrap().to_str().unwrap().to_string(),
        );
        params.insert("exec_id".to_string(), id.clone());
        let is_end = !cap[5].is_empty();
        let range = cap.get(0).unwrap().range();

        let tag_function = FunctionType::from(id, function, params, is_end, range);
        if let Some(last) = to_collection.last_mut() {
            if last.first().unwrap().id() == tag_function.id() {
                last.push(tag_function);
                continue;
            }
        }
        to_collection.push(vec![tag_function]);
    }
    let pairs: Vec<TagFunctionPair> = to_collection
        .into_iter()
        .map(|mut v| {
            if v.first().unwrap() == v.last().unwrap() {
                panic!("No end tag found for {}", v.first().unwrap().id());
            }
            if v.len() > 2 {
                panic!(
                    "ID {} has more than 2 tags - print {:?}",
                    v.first().unwrap().id(),
                    v
                );
            }
            TagFunctionPair {
                start: v.remove(0),
                end: v.remove(0),
            }
        })
        .collect();
    let mut location = 0;
    let mut processed: Vec<Chunk> = pairs
        .iter()
        .map(|pair| {
            let pair_clone = pair.clone();
            let identity_chunk = new_identity_chunk(location..pair.start.range().start, message);
            let tag_chunk = Chunk {
                pair: pair.clone(),
                text: message[pair.internal_range()].to_string(),
                opening_tag: message[pair.start.range().clone()].to_string(),
                closing_tag: message[pair.end.range().clone()].to_string(),
            };
            location = pair_clone.end.range().end;
            (identity_chunk, tag_chunk)
        })
        .flat_map(|(identity_chunk, tag_chunk)| vec![identity_chunk, tag_chunk])
        .collect();
    processed.push(new_identity_chunk(location..message.len(), message));
    processed
}

fn process_file(content: &str, id: Option<String>, file: PathBuf) {
    let chunks = process_to_chunks(content, &file);
    match chunks.len() {
        // If no chunks, do nothing
        0 => {}
        _ => {
            let file_content = chunks
                .iter()
                .map(|chunk| chunk.transform(id.clone()).print_representation())
                .collect::<Vec<String>>()
                .join("");
            std::fs::write(file, file_content).expect("Error writing to _file");
        }
    }
}

pub fn generate(path_str: &str, id: Option<String>) {
    let path = Path::new(path_str);
    let files = match path.is_file() || path.is_dir() {
        false => {
            panic!("Path argument must point to file or directory")
        }
        true => match path.is_file() {
            true => {
                vec![PathBuf::from(path_str)]
            }
            false => std::fs::read_dir(path_str)
                .unwrap()
                .map(|res| res.unwrap().path())
                .filter(|path| path.is_file())
                .filter(|path| path.extension().is_some_and(|x| x == "md"))
                .collect::<Vec<_>>(),
        },
    };
    for file in files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            process_file(&content, id.clone(), file);
        } else {
            panic!("Error reading file {}", file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_identity_chunk_test() {
        let message = "Hello world abcd";
        let range = 0..11;
        let chunk = new_identity_chunk(range.clone(), message);
        assert_eq!(chunk.pair.start.range(), range.start..range.start);
        assert_eq!(chunk.pair.end.range(), range.end..range.end);
        assert_eq!(chunk.text, "Hello world");
    }
    #[test]
    fn test_new_identity_chunk() {
        let range = 0..5;
        let message = "hello";
        let chunk = new_identity_chunk(range, message);
        assert_eq!(chunk.transform(None).text, "hello");
        assert_eq!(chunk.text, "hello");
    }

    #[test]
    fn test_chunk_run() {
        let range = 0..5;
        let message = "hello";
        let chunk = new_identity_chunk(range, message);
        let x = chunk.transform(None);
        assert_eq!(x.text, "hello");
    }

    #[test]
    fn test_chunk_print_representation() {
        let range = 0..5;
        let message = "hello";
        let chunk = new_identity_chunk(range, message);
        let x = chunk.transform(None);
        assert_eq!(x.print_representation(), "hello");
    }
}

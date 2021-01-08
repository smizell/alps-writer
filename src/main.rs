extern crate comrak;
use comrak::{markdown_to_html, ComrakOptions};
use serde::{de, Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::error::Error;
use std::format;
use std::fs;
use std::path::Path;
// use walkdir::WalkDir;

fn main() {
    let profile_dir = Path::new("example/profile");
    let alps = walk_profile(&profile_dir).unwrap();
    println!("{:?}", alps);
}

fn walk_profile(profile_dir: &Path) -> Result<Alps, &str> {
    let index = profile_dir.join("index.md");
    if !index.exists() {
        return Err("No index found in profile directory");
    }
    let alps = Alps::from_file(&index).unwrap();
    return Ok(alps);
}

fn walk_dir(path: &Path) {
    // Get the list of directories
    for entry in path.read_dir().expect("read_dir failed") {
        if let Ok(entry) = entry {
            if entry.path().is_file() && entry.file_name() == "index.md" {
                let data = fs::read_to_string(entry.path()).expect("Unable to read index.md");
                let parts: Vec<&str> = data.split("---").collect();
                if parts.len() < 3 {
                    println!("Frontmatter not correct in {:?}", entry.path());
                    continue;
                }
                let frontmatter: Alps = serde_yaml::from_str(&parts[1]).unwrap();
                let body = markdown_to_html(&parts[2], &ComrakOptions::default());
                println!("{:?}", frontmatter);
            }

            if entry.path().is_dir() {
                println!("{:?}", entry.path());
                walk_dir(&entry.path());
            }
        }
    }
}

fn read_markdown_file<T>(path: &Path) -> Result<T, &'static str>
where
    T: de::DeserializeOwned + WithDoc,
{
    let data = fs::read_to_string(path).unwrap();
    let parts: Vec<&str> = data.split("---").collect();
    if parts.len() < 3 {
        return Err("Can't format error");
    }
    let mut result: T = serde_yaml::from_str(&parts[1]).unwrap();
    let value = markdown_to_html(&parts[2], &ComrakOptions::default());
    result.add_markdown_doc(value);
    return Ok(result);
}

trait WithDoc {
    fn add_markdown_doc<'a>(self: &'a mut Self, value: String) -> &'a mut Self;
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Alps {
    descriptor: Option<Vec<Descriptor>>,
    doc: Option<Doc>,
    link: Option<Vec<Link>>,
}

impl Alps {
    fn from_file(path: &Path) -> Result<Alps, &'static str> {
        let alps = read_markdown_file::<Alps>(path).unwrap();
        return Ok(alps);
    }
}

impl WithDoc for Alps {
    fn add_markdown_doc<'a>(&'a mut self, value: String) -> &'a mut Alps {
        self.doc = Some(Doc {
            format: String::from("markdown"),
            value,
        });
        self
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Link {
    rel: String,
    href: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Descriptor {
    id: Option<String>,
    name: Option<String>,
    rel: Option<String>,
    rt: Option<String>,
    link: Vec<Link>,

    #[serde(rename(serialize = "type"))]
    #[serde(rename(deserialize = "type"))]
    descriptor_type: DescriptorType,

    descriptor: Vec<Descriptor>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum DescriptorType {
    Idempotent,
    Semantic,
    Safe,
    Unsafe,
}

impl Default for DescriptorType {
    fn default() -> Self {
        DescriptorType::Semantic
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Doc {
    #[serde(default = "default_format")]
    format: String,
    value: String,
}

fn default_format() -> String {
    String::from("markdown")
}

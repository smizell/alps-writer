extern crate comrak;
use comrak::{markdown_to_html, ComrakOptions};
use serde::{de, Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::error::Error;
use std::format;
use std::fs::{self, DirEntry};
use std::io;
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
    let mut alps = Alps::from_file(&index).unwrap();

    for entry in profile_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_dir() && path.file_name().unwrap() != "index.md" {
            println!("{:?}", entry.path());
            let desc = Descriptor::from_file(&entry.path()).unwrap();
            alps.add_descriptor(desc);
            continue;
        }
    }

    Ok(alps)
}

fn read_markdown_file<T>(path: &Path) -> Result<T, &'static str>
where
    T: de::DeserializeOwned + WithDoc,
{
    let data = fs::read_to_string(path).unwrap();
    let parts: Vec<&str> = data.split("---").collect();

    // TODO: Make it work with 1 length
    if parts.len() < 3 {
        return Err("Can't format error");
    }

    let frontmatter = if parts[1].trim().is_empty() {
        String::from("{}")
    } else {
        parts[1].to_string()
    };

    let mut result: T = serde_yaml::from_str(&frontmatter).unwrap();
    result.add_markdown_doc(parts[2].to_string());
    return Ok(result);
}

trait WithDoc {
    fn add_markdown_doc<'a>(self: &'a mut Self, value: String) -> &'a mut Self;
}

fn default_descriptor() -> Vec<Descriptor> {
    vec![]
}

fn default_descriptor_type() -> DescriptorType {
    DescriptorType::Semantic
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Alps {
    #[serde(default = "default_descriptor")]
    descriptor: Vec<Descriptor>,
    doc: Option<Doc>,
    link: Option<Vec<Link>>,
}

impl Alps {
    fn from_file(path: &Path) -> Result<Alps, &'static str> {
        let alps = read_markdown_file::<Alps>(path).unwrap();
        return Ok(alps);
    }

    fn add_descriptor<'a>(&'a mut self, descriptor: Descriptor) -> &'a mut Alps {
        self.descriptor.push(descriptor);
        self
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
    link: Option<Vec<Link>>,
    doc: Option<Doc>,

    #[serde(rename(serialize = "type"))]
    #[serde(rename(deserialize = "type"))]
    #[serde(default = "default_descriptor_type")]
    descriptor_type: DescriptorType,

    #[serde(default = "default_descriptor")]
    descriptor: Vec<Descriptor>,
}

impl Descriptor {
    fn from_file(path: &Path) -> Result<Descriptor, &'static str> {
        let mut descriptor = read_markdown_file::<Descriptor>(path).unwrap();
        let desc_id = path.file_stem().unwrap().to_str().unwrap();
        descriptor.id = Some(desc_id.to_string());
        Ok(descriptor)
    }
}

impl WithDoc for Descriptor {
    fn add_markdown_doc<'a>(&'a mut self, value: String) -> &'a mut Descriptor {
        self.doc = Some(Doc {
            format: String::from("markdown"),
            value,
        });
        self
    }
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

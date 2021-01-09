// extern crate comrak;
// use comrak::{markdown_to_html, ComrakOptions};
// use std::collections::HashMap;
// use std::error::Error;
// use std::format;
// use std::io;
// use walkdir::WalkDir;
use serde::{de, Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::fs;
use std::path::Path;

fn main() {
    let profile_dir = Path::new("example/profile");
    let alps = walk_profile::<Alps>(&profile_dir).unwrap();
    let alps_document = AlpsDocument { alps };
    let s = serde_json::to_string_pretty(&alps_document).unwrap();
    println!("{}", s);
}

fn walk_profile<T>(profile_dir: &Path) -> Result<T, &str>
where
    T: FromFile + WithDescriptor,
{
    let index = profile_dir.join("index.md");
    if !index.exists() {
        return Err("No index found in profile directory");
    }
    let mut main = T::from_file(&index).unwrap();

    for entry in profile_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let descriptor = if !path.is_dir() && path.file_name().unwrap() != "index.md" {
            // Local .md files
            // We processed index.md above so we can skip it
            Descriptor::from_file(&entry.path()).unwrap()
        } else if path.is_dir() {
            // Handles folders that are treated like Descriptors
            walk_profile::<Descriptor>(&path).unwrap()
        } else {
            continue;
        };

        main.add_descriptor(descriptor);
    }

    Ok(main)
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
    result.add_doc(String::from("markdown"), parts[2].to_string());
    return Ok(result);
}

trait WithDoc {
    fn add_doc<'a>(self: &'a mut Self, format: String, value: String) -> &'a mut Self;
}

trait FromFile {
    fn from_file(path: &Path) -> Result<Self, &'static str>
    where
        Self: Sized;
}

trait WithDescriptor {
    fn add_descriptor<'a>(self: &'a mut Self, descriptor: Descriptor) -> &'a mut Self;
}

fn default_descriptor() -> Vec<Descriptor> {
    vec![]
}

fn default_descriptor_type() -> DescriptorType {
    DescriptorType::Semantic
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct AlpsDocument {
    alps: Alps,
}

#[derive(Debug, Deserialize, Serialize)]
struct Alps {
    #[serde(default = "default_descriptor")]
    descriptor: Vec<Descriptor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<Doc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<Vec<Link>>,
}

impl Default for Alps {
    fn default() -> Alps {
        Alps {
            descriptor: vec![],
            doc: None,
            link: None,
        }
    }
}

impl FromFile for Alps {
    fn from_file(path: &Path) -> Result<Alps, &'static str> {
        let alps = read_markdown_file::<Alps>(path).unwrap();
        return Ok(alps);
    }
}

impl WithDescriptor for Alps {
    fn add_descriptor<'a>(&'a mut self, descriptor: Descriptor) -> &'a mut Alps {
        self.descriptor.push(descriptor);
        self
    }
}

impl WithDoc for Alps {
    fn add_doc<'a>(&'a mut self, format: String, value: String) -> &'a mut Alps {
        self.doc = Some(Doc { format, value });
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
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<Vec<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<Doc>,

    #[serde(rename(serialize = "type"))]
    #[serde(rename(deserialize = "type"))]
    #[serde(default = "default_descriptor_type")]
    descriptor_type: DescriptorType,

    #[serde(default = "default_descriptor")]
    descriptor: Vec<Descriptor>,
}

impl FromFile for Descriptor {
    fn from_file(path: &Path) -> Result<Descriptor, &'static str> {
        let mut descriptor = read_markdown_file::<Descriptor>(path).unwrap();
        let desc_id = path.file_stem().unwrap().to_str().unwrap();
        descriptor.id = Some(desc_id.to_string());
        Ok(descriptor)
    }
}

impl WithDescriptor for Descriptor {
    fn add_descriptor<'a>(&'a mut self, descriptor: Descriptor) -> &'a mut Descriptor {
        self.descriptor.push(descriptor);
        self
    }
}

impl WithDoc for Descriptor {
    fn add_doc<'a>(&'a mut self, format: String, value: String) -> &'a mut Descriptor {
        self.doc = Some(Doc { format, value });
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

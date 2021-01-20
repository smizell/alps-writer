#[macro_use]
extern crate clap;
use serde::{de, Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::fs;
use std::path::Path;
use std::io::prelude::*;

fn main() {
    let matches = clap_app!(alps_writer_cli =>
        (version: "0.1")
        (about: "Tools for writing ALPS profiles")
        (@subcommand profile =>
            (about: "Builds ALPS document from profile directory")
            (@arg DIR: +required "Sets the base directory for the profile")
        )
        (@subcommand descriptor =>
            (about: "Create a new descriptor")
            (@arg DESCRIPTOR_PATH: +required "Path for the new descriptor (do not add .md)")
        )
    )
    .get_matches();

    if let Some(matches) = matches.subcommand_matches("profile") {
        let dir = matches.value_of("DIR").unwrap();
        let profile_dir = Path::new(&dir);
        build_profile(&profile_dir)
    }

    if let Some(matches) = matches.subcommand_matches("descriptor") {
        let desc_path = matches.value_of("DESCRIPTOR_PATH").unwrap();

        let desc_dir_path = Path::new(&desc_path);
        let file_path_value = format!("{}.md", &desc_path);
        let desc_file_path = Path::new(&file_path_value);

        // check for existing directory or .md file
        // error if either exists
        if desc_dir_path.exists() || desc_file_path.exists() {
            panic!("It looks like the descriptor already exists.")
        }

        // check for parent .md file
        // if exists, convert to directory, move to index.md
        // create new .md file for descriptor
        // let full_desc_dir_path = profile_dir.join(desc_dir_path);
        let parent_dir = desc_dir_path.parent().unwrap();
        let parent_md_name = format!("{}.md", parent_dir.to_str().unwrap());
        let parent_md_path = Path::new(&parent_md_name);
        if parent_md_path.exists() {
            fs::create_dir(parent_dir).unwrap();
            fs::rename(parent_md_path, parent_dir.join("index.md")).unwrap();
        }

        // check for existing parent directory
        // if no parent directory, error
        // if parent directory, create new .md file
        if desc_dir_path.parent().unwrap().exists() {
            let mut new_descriptor_file = fs::File::create(desc_file_path).unwrap();
            new_descriptor_file.write_all(b"---\n---\n\n").unwrap();
        } else {
            panic!("Parent descriptor does not exist. Please create it first.");
        }
    }
}

fn build_profile(path: &Path) {
    let alps = walk_profile::<Alps>(&path).unwrap();
    let alps_document = AlpsDocument { alps };
    let s = serde_json::to_string_pretty(&alps_document).unwrap();
    println!("{}", s);
}

// fn create_descriptor()

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
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if file_name == "index.md" {
            continue;
        }

        let descriptor = if !path.is_dir() && file_name.ends_with(".md") {
            // Local .md files
            // We processed index.md above so we can skip it
            Descriptor::from_file(&entry.path()).unwrap()
        } else if path.is_dir() {
            // Handles folders that are treated like Descriptors
            let mut desc_walked = walk_profile::<Descriptor>(&path).unwrap();
            desc_walked.id = Some(path.file_name().unwrap().to_str().unwrap().to_string());
            desc_walked
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

    // This allows us to handle files with or without frontmatter along with empty frontmatter and body.
    let (frontmatter, body) = match parts.len() {
        1 => (String::from("{}"), parts[0].trim()),
        3 => match parts[1].trim().is_empty() {
            // We have to pass in something for serde_yaml, so we do {} if empty
            true => (String::from("{}"), parts[2].trim()),
            false => (String::from(parts[1].trim()), parts[2].trim()),
        },
        _ => return Err("Can't handle file format"),
    };

    let mut result: T = serde_yaml::from_str(&frontmatter).unwrap();

    if !body.is_empty() {
        result.add_doc(String::from("markdown"), body.to_string());
    }

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

fn default_descriptor_type() -> DescriptorType {
    DescriptorType::Semantic
}

fn default_version() -> String {
    String::from("1.0")
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct AlpsDocument {
    alps: Alps,
}

#[derive(Debug, Deserialize, Serialize)]
struct Alps {
    #[serde(default = "default_version")]
    version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    descriptor: Option<Vec<Descriptor>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<Doc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<Vec<Link>>,
}

impl Default for Alps {
    fn default() -> Alps {
        Alps {
            version: String::from("1.0"),
            descriptor: Some(vec![]),
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
        match self.descriptor {
            Some(ref mut d) => d.push(descriptor),
            None => self.descriptor = Some(vec![descriptor])
        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Descriptor {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link: Option<Vec<Link>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<Doc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    def: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    #[serde(rename(serialize = "type"))]
    #[serde(rename(deserialize = "type"))]
    #[serde(default = "default_descriptor_type")]
    descriptor_type: DescriptorType,

    // #[serde(default = "default_descriptor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    descriptor: Option<Vec<Descriptor>>,
}

impl FromFile for Descriptor {
    fn from_file(path: &Path) -> Result<Descriptor, &'static str> {
        let mut descriptor = read_markdown_file::<Descriptor>(path).unwrap();

        // This lets people overwrite the ID from the file
        if let None = descriptor.id {
            let desc_id = path.file_stem().unwrap().to_str().unwrap();
            descriptor.id = Some(desc_id.to_string());
        }

        Ok(descriptor)
    }
}

impl WithDescriptor for Descriptor {
    fn add_descriptor<'a>(&'a mut self, descriptor: Descriptor) -> &'a mut Descriptor {
        match self.descriptor {
            Some(ref mut d) => d.push(descriptor),
            None => self.descriptor = Some(vec![descriptor])
        }
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

use std::{
    fs,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use crate::error::Error;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use toml::value::{Date as TomlDate, Datetime as TomlDatetime};

// Used to separate the "top" of the post, to be used as a summary.
// Currently not used for anything, maybe a tooltip or summary page later.
const TOP_TAG: &str = "<!-- top -->";

pub struct Posts {
    root: PathBuf,
    posts: Vec<Post>,
    tags: Vec<String>,
}

#[derive(Clone)]
pub struct Post {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub content: String,
    pub top: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct PostHeader {
    title: String,
    date: TomlDatetime,
    tags: Vec<String>,
}

impl Posts {
    pub fn new(root: &Path) -> Result<Self, Error> {
        let posts = collect_posts(root)?;
        let tags = collect_tags(&posts);
        Ok(Self {
            root: root.into(),
            posts,
            tags,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn create_post(&mut self) -> Result<Post, Error> {
        let next = self.posts.len();
        let date = Utc::today().naive_utc();

        let post = Post {
            name: next.to_string(),
            title: String::new(),
            date,
            tags: Vec::new(),
            content: String::new(),
            top: None,
        };

        let post_path = &self.root.join(&post.name).with_extension("md");
        let header = PostHeader {
            title: String::new(),
            date: TomlDatetime {
                date: Some(TomlDate {
                    year: date.year() as u16,
                    month: date.month() as u8,
                    day: date.day() as u8,
                }),
                time: None,
                offset: None,
            },
            tags: Vec::new(),
        };

        let header = toml::to_string(&header).expect("Failed to serialize post header");
        println!("» created new post '{}'", post_path.display());
        println!("{}", header);
        fs::write(post_path, format!("---\n{}---\n\n{}", header, TOP_TAG))?;

        self.posts.push(post.clone());
        Ok(post)
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }
}

impl Deref for Posts {
    type Target = Vec<Post>;

    fn deref(&self) -> &Self::Target {
        &self.posts
    }
}

impl DerefMut for Posts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.posts
    }
}

impl Post {
    fn read(root: &Path, path: &Path) -> Result<Self, Error> {
        let contents = fs::read_to_string(root.join(path))?;
        let header_end = contents[4..].find("---").expect("invalid post header") + 4;
        let toml = &contents[4..header_end];

        let PostHeader { title, date, tags } =
            toml::from_str(toml).map_err(|e| Error::ReadPostHeader(path.into(), e))?;
        let date = date.date.expect("expected TOML date");
        let content = &contents[header_end + 5..];
        let top = content.find(TOP_TAG);
        let name = path
            .file_stem()
            .expect("expected file name")
            .to_str()
            .expect("expected UTF-8")
            .to_string();

        Ok(Self {
            name,
            title,
            date: NaiveDate::parse_from_str(&date.to_string(), "%Y-%m-%d")?,
            tags,
            content: content.into(),
            top,
        })
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn rss_date(&self) -> String {
        let dt = Utc.from_local_date(&self.date).unwrap().and_hms(0, 0, 0);
        dt.to_rfc2822()
    }
}

fn collect_posts(root: &Path) -> Result<Vec<Post>, Error> {
    let mut posts = Vec::new();
    for entry in root.read_dir()? {
        let path = entry?.path();
        if !path.is_dir() {
            if let Some(name) = path.file_name() {
                posts.push(Post::read(root, Path::new(name))?);
            }
        }
    }
    posts.sort_by_key(|post| format!("{}-{}", &post.date, &post.title));
    posts.reverse();
    Ok(posts)
}

fn collect_tags(posts: &[Post]) -> Vec<String> {
    let mut tags = Vec::new();
    for post in posts {
        for tag in &post.tags {
            tags.push(tag.to_string());
        }
    }
    tags.sort_unstable();
    tags.dedup();
    tags
}

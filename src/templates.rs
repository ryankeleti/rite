use std::path::Path;

use askama::Template;

use crate::post::{Post, Posts};

mod filters {
    use std::path::Path;

    pub fn path(path: &Path) -> askama::Result<String> {
        Ok(path.display().to_string().trim_end_matches('/').to_string())
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    // Document title element.
    pub title: &'a str,
    // Posts root directory.
    pub posts_root: &'a Path,
    // Index content.
    pub content: &'a str,
}

/// RSS feed template for posts.
#[derive(Template)]
#[template(path = "rss.xml")]
pub struct RssTemplate<'a> {
    // RSS title.
    pub title: &'a str,
    // Full base URL for the posts.
    pub posts_url: &'a str,
    // RSS description.
    pub description: &'a str,
    // Posts to be included.
    pub posts: &'a Posts,
}

/// Individual post template.
#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Posts root directory.
    pub posts_root: &'a Path,
    // Post to be rendered.
    pub post: &'a Post,
    // Additional scripts.
    // Used for scripts needed only for posts.
    pub scripts: &'a ScriptsTemplate,
}

/// Posts index template.
#[derive(Template)]
#[template(path = "posts.html")]
pub struct PostsTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Description about the blog.
    pub description: &'a str,
    // Posts to be included.
    pub posts: &'a Posts,
}

/// Post tag template.
#[derive(Template)]
#[template(path = "tag.html")]
pub struct TagTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Name of the tag.
    pub name: &'a str,
    // Posts to be searched for tags.
    pub posts: &'a Posts,
}

/// Post tags index template.
#[derive(Template)]
#[template(path = "tags.html")]
pub struct TagsTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Posts root directory.
    pub posts_root: &'a Path,
    // List of tags.
    pub tags: &'a [&'a str],
}

/// 404 not found template.
#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Not found user message.
    pub message: &'a str,
}

/// A generic content page.
#[derive(Template)]
#[template(path = "content.html")]
pub struct ContentTemplate<'a> {
    // Document (base) title element.
    pub title: &'a str,
    // Name of the content, to be included after title.
    pub name: &'a str,
    // HTML content of the page.
    pub content: &'a str,
}

/// A list of script elements, with an optional noscript element.
#[derive(Template)]
#[template(path = "scripts.html")]
pub struct ScriptsTemplate {
    pub scripts: Vec<Script>,
    pub noscript: Option<String>,
}

/// Simple script element.
pub enum Script {
    /// An embedded script.
    Embed { contents: String },
    /// A script from an external source.
    /// Uses `async`.
    Src { src: String },
}

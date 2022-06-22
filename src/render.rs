use std::{fs, path::Path};

use askama::Template;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};

use crate::{
    config::Config,
    error::Error,
    post::{Post, Posts},
    templates::{
        ContentTemplate, IndexTemplate, NotFoundTemplate, PostTemplate, PostsTemplate, RssTemplate,
        Script, ScriptsTemplate, TagTemplate, TagsTemplate,
    },
};

// File stems in the `config.content` directory reserved for specific templates.
const RESERVED_CONTENT_NAMES: &[&str] = &["index", "posts", "404"];

// Fallback if no theme is specified in `config.toml`.
//
// Can replace with available options listed in
// <https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html#method.load_defaults>.
const DEFAULT_SYNTAX_THEME: &str = "base16-mocha.dark";

/// Renderer for the site's content.
pub struct Renderer<'a> {
    config: &'a Config,
    markdown: Markdown,
}

impl<'a> Renderer<'a> {
    pub fn new(config: &'a Config) -> Result<Self, Error> {
        let theme = match &config.syntax_theme {
            Some(path) => {
                ThemeSet::get_theme(path).map_err(|e| Error::SyntectLoad(path.clone(), e))?
            }
            None => {
                let ts = ThemeSet::load_defaults();
                ts.themes[DEFAULT_SYNTAX_THEME].clone()
            }
        };
        Ok(Self {
            config,
            markdown: Markdown::with_theme(theme),
        })
    }

    /// Renders main content and posts/tags.
    pub fn render(&self) -> Result<(), Error> {
        self.render_index()?;
        self.render_not_found()?;
        self.render_content()?;

        let posts = Posts::new(&self.config.posts)?;
        self.render_posts_and_tags(posts)?;

        Ok(())
    }

    // Render index page.
    fn render_index(&self) -> Result<(), Error> {
        let src = self.config.content.join("index.md");
        let content = &self.content_or_blank(&src)?;
        let template = IndexTemplate {
            title: &self.config.title,
            posts_root: &self.config.posts_root,
            content,
        };

        let dest = self.config.build_root.join("index.html");
        let render = template.render()?;

        println!(">> creating '{}'", dest.display());
        fs::write(dest, render)?;
        Ok(())
    }

    // Render not found page.
    fn render_not_found(&self) -> Result<(), Error> {
        let src = self.config.content.join("404.md");
        let message = &self.content_or_blank(&src)?;
        let template = NotFoundTemplate {
            title: &self.config.title,
            message,
        };

        let dest = self.config.build_root.join("404.html");
        let render = template.render()?;

        println!(">> creating '{}'", dest.display());
        fs::write(dest, render)?;
        Ok(())
    }

    // Render additional content pages.
    fn render_content(&self) -> Result<(), Error> {
        println!(">> creating additional content");
        for entry in self.config.content.read_dir()? {
            let path = entry?.path();
            if !path.is_dir() {
                // TODO. Walk recursively?
                // Also maybe clean it up.
                let name = path.file_stem().unwrap().to_str().unwrap();
                if !RESERVED_CONTENT_NAMES.contains(&name) {
                    let content = fs::read_to_string(&path)?;
                    let content = &self.markdown.render_html(&content)?;
                    let template = ContentTemplate {
                        title: &self.config.title,
                        name,
                        content,
                    };
                    let dest = self.config.build_root.join(name).with_extension("html");
                    let render = template.render()?;
                    println!("  -- '{}'", dest.display());
                    fs::write(dest, render)?;
                }
            }
        }
        Ok(())
    }

    // Render posts, tags, and RSS feed.
    fn render_posts_and_tags(&self, mut posts: Posts) -> Result<(), Error> {
        let posts_dir = self.config.build_root.join(&self.config.posts_root);
        if !posts_dir.exists() {
            println!(
                ">> creating posts build directory '{}'",
                posts_dir.display()
            );
            fs::create_dir_all(&posts_dir)?;
        }

        // Create posts index.
        let posts_src = self.config.content.join("posts.md");
        let description = &self.content_or_blank(&posts_src)?;
        let posts_template = PostsTemplate {
            title: &self.config.title,
            description,
            posts: &posts,
        };

        let posts_dest = posts_dir.join("index.html");
        let posts_render = posts_template.render()?;

        println!(">> creating '{}'", posts_dest.display());
        fs::write(posts_dest, posts_render)?;

        let scripts = self.get_post_scripts()?;

        for post in posts.iter_mut() {
            post.content = self.markdown.render_html(&post.content)?;
            println!("  -- rendering post '{}'", post.name);
            self.render_post(post, &scripts)?;
        }

        println!(">> rendering RSS");
        self.render_rss(&posts)?;

        let tags_dir = posts_dir.join("tags");
        if !tags_dir.exists() {
            println!(">> creating tags directory '{}'", tags_dir.display());
            fs::create_dir_all(&tags_dir)?;
        }

        let tags: Vec<_> = posts.tags().iter().map(|tag| tag.as_ref()).collect();

        // Create tags index.
        let tags_template = TagsTemplate {
            title: &self.config.title,
            posts_root: &self.config.posts_root,
            tags: &tags,
        };

        let tags_dest = tags_dir.join("index.html");
        let tags_render = tags_template.render()?;

        println!(">> creating '{}'", tags_dest.display());
        fs::write(tags_dest, tags_render)?;

        for tag in &tags {
            println!("  -- rendering tag '{}'", tag);
            self.render_tag(tag, &posts)?;
        }
        Ok(())
    }

    fn render_post(&self, post: &Post, scripts: &ScriptsTemplate) -> Result<(), Error> {
        let dest = self
            .config
            .build_root
            .join(&self.config.posts_root)
            .join(&post.name)
            .with_extension("html");
        let template = PostTemplate {
            title: &self.config.title,
            posts_root: &self.config.posts_root,
            post,
            scripts,
        };
        let render = template.render()?;
        fs::write(dest, render)?;
        Ok(())
    }

    fn render_rss(&self, posts: &Posts) -> Result<(), Error> {
        let dest = self
            .config
            .build_root
            .join(&self.config.posts_root)
            .join("rss")
            .with_extension("xml");
        let template = RssTemplate {
            title: &self.config.title,
            posts_url: &format!("{}/{}", self.config.url, self.config.posts_root.display()),
            description: &format!("{} posts", self.config.title),
            posts,
        };
        let render = template.render()?;
        fs::write(dest, render)?;
        Ok(())
    }

    fn render_tag(&self, tag: &str, posts: &Posts) -> Result<(), Error> {
        let dest = self
            .config
            .build_root
            .join(&self.config.posts_root)
            .join("tags")
            .join(tag)
            .with_extension("html");
        let template = TagTemplate {
            title: &self.config.title,
            name: tag,
            posts,
        };
        let render = template.render()?;
        fs::write(dest, render)?;
        Ok(())
    }

    fn content_or_blank(&self, path: &Path) -> Result<String, Error> {
        Ok(if path.exists() {
            let content = fs::read_to_string(path)?;
            self.markdown.render_html(&content)?
        } else {
            String::new()
        })
    }

    fn get_post_scripts(&self) -> Result<ScriptsTemplate, Error> {
        let mut scripts = Vec::new();
        match &self.config.posts_embed_scripts {
            Some(path) => {
                for entry in path.read_dir()? {
                    let path = entry?.path();
                    if !path.is_dir() {
                        let contents = fs::read_to_string(&path)?;
                        scripts.push(Script::Embed { contents });
                    }
                }
            }
            None => (),
        }

        match &self.config.posts_src_scripts {
            Some(srcs) => {
                for src in srcs {
                    scripts.push(Script::Src {
                        src: src.to_string(),
                    });
                }
            }
            None => (),
        }

        Ok(ScriptsTemplate {
            scripts,
            noscript: self.config.posts_noscript.clone(),
        })
    }
}

struct Markdown {
    syntax_set: SyntaxSet,
    theme: Theme,
    options: Options,
}

impl Markdown {
    fn with_theme(theme: Theme) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_FOOTNOTES);
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme,
            options,
        }
    }

    fn render_html(&self, content: &str) -> Result<String, Error> {
        let parser = Parser::new_ext(content, self.options);
        let events = syntax_hl(parser, &self.syntax_set, &self.theme)?;
        let events = notes(events);
        let mut html = String::new();
        html::push_html(&mut html, events.into_iter());
        Ok(html)
    }
}

fn syntax_hl<'a>(
    events: impl Iterator<Item = Event<'a>>,
    syntax_set: &SyntaxSet,
    theme: &Theme,
) -> Result<Vec<Event<'a>>, Error> {
    let mut result = Vec::new();
    let mut to_highlight = String::new();
    let mut in_code_block = false;
    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(Tag::CodeBlock(lang)) if in_code_block => {
                let lang = match lang {
                    CodeBlockKind::Fenced(s) => s.into_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let syntax = if lang.is_empty() {
                    syntax_set.find_syntax_plain_text()
                } else {
                    match syntax_set.find_syntax_by_extension(&lang) {
                        Some(syn) => syn,
                        None => syntax_set.find_syntax_plain_text(),
                    }
                };
                let html = syntect::html::highlighted_html_for_string(
                    &to_highlight,
                    syntax_set,
                    syntax,
                    theme,
                )
                .map_err(Error::Syntect)?;
                result.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                to_highlight = String::new();
                in_code_block = false;
            }
            Event::Text(t) => {
                if in_code_block {
                    to_highlight.push_str(&t);
                } else {
                    result.push(Event::Text(t));
                }
            }
            _ => result.push(event),
        }
    }
    Ok(result)
}

fn notes(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut result = Vec::new();
    let mut in_note = false;
    let mut in_sidenote = false;
    let mut sidenotes = collect_sidenotes(&events);
    for event in events {
        match event {
            Event::FootnoteReference(ref label) => {
                if label.starts_with('s') {
                    if let Some(note) = sidenotes.pop() {
                        let html = format!(
                            r#"<span class="sidenote-number"><small class="sidenote">{}</small></span>"#,
                            note
                        );
                        result.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                    }
                } else {
                    result.push(event);
                }
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                in_note = true;
                if !label.starts_with('s') {
                    let html = format!("<p><sup>{}</sup> ", label);
                    result.push(Event::Html(CowStr::Boxed(html.into_boxed_str())));
                } else {
                    in_sidenote = true;
                }
            }
            Event::End(Tag::FootnoteDefinition(label)) => {
                in_note = false;
                if !label.starts_with('s') {
                    result.push(Event::Html(CowStr::Borrowed("</p>")));
                } else {
                    in_sidenote = false;
                }
            }
            Event::Start(Tag::Paragraph) | Event::End(Tag::Paragraph) if in_note => (),
            Event::Text(_) if in_sidenote => (),
            _ => result.push(event),
        };
    }
    result
}

fn collect_sidenotes(events: &[Event<'_>]) -> Vec<String> {
    let mut sidenotes = Vec::new();
    let mut buf = String::new();
    let mut in_note = false;
    for event in events.iter() {
        match event {
            Event::Start(Tag::FootnoteDefinition(_)) => {
                in_note = true;
            }
            Event::End(Tag::FootnoteDefinition(_)) => {
                in_note = false;
                sidenotes.push(buf.clone());
                buf.clear();
            }
            Event::Text(t) if in_note => buf.push_str(&t.clone().into_string()),
            Event::SoftBreak if in_note => buf += " ",
            _ => (),
        }
    }
    sidenotes.reverse();
    sidenotes
}

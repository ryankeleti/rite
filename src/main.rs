use std::{env, fs, path::Path};

mod config;
mod error;
mod post;
mod render;
mod templates;
mod util;

use config::Config;
use error::Error;
use post::Posts;
use render::Renderer;

const STATIC_FILES_PATH: &str = "static";

fn main() {
    match wrap_error() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn wrap_error() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let config = config::read_config()?;

    match handle_args(&args) {
        Args::Build => build(&config)?,
        Args::Post => new_post(&config)?,
        Args::Missing => {
            eprintln!("usage: {} build | post", &args[0]);
            std::process::exit(1);
        }
        Args::Unknown(s) => {
            eprintln!("unknown command '{}'.", s);
            eprintln!("usage: {} build | post", &args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_args(args: &[String]) -> Args {
    match args.get(1).map(|s| &s[..]) {
        Some("b" | "build") => Args::Build,
        Some("p" | "post") => Args::Post,
        Some(s) => Args::Unknown(s.into()),
        None => Args::Missing,
    }
}

enum Args {
    Build,
    Post,

    // Errors.
    Missing,
    Unknown(String),
}

fn build(config: &Config) -> Result<(), Error> {
    if config.build_root.exists() {
        println!(
            ">> removing build directory '{}'",
            config.build_root.display()
        );
        fs::remove_dir_all(&config.build_root)?;
    }

    println!(
        ">> creating build directory '{}'",
        config.build_root.display()
    );
    fs::create_dir_all(&config.build_root)?;

    println!(">> copying static files");
    let static_dir = Path::new(STATIC_FILES_PATH);
    util::copy_static(static_dir, &config.build_root.join(static_dir))?;

    let renderer = Renderer::new(config)?;
    renderer.render()?;

    Ok(())
}

fn new_post(config: &Config) -> Result<(), Error> {
    let mut posts = Posts::new(&config.posts)?;
    posts.create_post()?;
    Ok(())
}

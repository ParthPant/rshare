mod cli;
mod templates;
mod utils;

use tokio::fs;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use warp::path::FullPath;
use warp::Filter;

use pretty_env_logger;
use tera::Context;

use clap::Parser;

use cli::Args;
use templates::Templates;
use utils::{decode_url, with_clone, ArcPath};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = Args::parse();

    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let root_dir = std::fs::canonicalize(PathBuf::from(config.dir.as_str())).unwrap();

    log::info!("serving directory: {}", root_dir.to_str().unwrap());

    let root_dir = ArcPath::from(root_dir);
    let templates = Arc::new(Templates::new());

    let not_found = warp::get()
        .and(warp::any())
        .and(warp::path::full())
        .and(with_clone(&templates))
        .map(move |path: FullPath, tera: Arc<Templates>| {
            let mut context = Context::new();
            context.insert("path", path.as_str());
            let html = tera.render("404.html", &context).unwrap();
            warp::reply::html(html)
        });

    let folders = warp::get()
        .and(warp::path::full())
        .and(with_clone(&templates))
        .and(with_clone(&root_dir))
        .and_then(
            |reqpath: FullPath, tera: Arc<Templates>, home_dir: ArcPath| async move {
                let reqpath = decode_url(reqpath.as_str().strip_prefix('/').unwrap());
                let reqpath = std::path::Path::new(&reqpath);
                let path = home_dir.0.join(&reqpath);
                let mut parent = std::path::Path::new(reqpath).to_owned();
                parent.pop();

                if path.is_dir() {
                    if let Ok(mut entries) = fs::read_dir(path.clone()).await {
                        let mut dirs: Vec<String> = Vec::new();
                        let mut files: Vec<String> = Vec::new();

                        while let Ok(entry) = entries.next_entry().await {
                            if let Some(entry) = entry {
                                if let Ok(entrytype) = entry.file_type().await {
                                    let name = entry.file_name().into_string().unwrap();
                                    if entrytype.is_dir() {
                                        dirs.push(name);
                                    } else {
                                        files.push(name);
                                    }
                                }
                            } else {
                                break;
                            }
                        }

                        let foldername = match path.file_name() {
                            Some(f) => f.to_str().unwrap(),
                            None => "/",
                        };

                        let mut context = Context::new();
                        context.insert("files", &files);
                        context.insert("dirs", &dirs);
                        context.insert("parent", &parent.to_str().unwrap());
                        context.insert("isroot", &(reqpath.to_str().unwrap() == ""));
                        context.insert("foldername", foldername);

                        log::debug!("reqpath: {:?}", reqpath);
                        log::debug!("path: {:?}", path);
                        log::debug!("parent: {:?}", parent);
                        log::debug!("foldername: {:?}", foldername);

                        let html = tera.render("list.html", &context).unwrap();
                        Ok(warp::reply::html(html))
                    } else {
                        Err(warp::reject::not_found())
                    }
                } else {
                    Err(warp::reject::not_found())
                }
            },
        );

    let files = warp::get().and(warp::any()).and(warp::fs::dir(root_dir));

    let routes = folders.or(files).or(not_found).with(warp::log("requests"));

    warp::serve(routes).run(([0, 0, 0, 0], config.port)).await;

    Ok(())
}

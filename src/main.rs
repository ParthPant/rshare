mod templates;
mod utils;

use tokio::fs;

use std::env;
use std::error::Error;
use std::sync::Arc;
use std::path::PathBuf;

use warp::Filter;
use warp::path::FullPath;

use tera::Context;
use pretty_env_logger;

use templates::Templates;
use utils::{with_clone, decode_url};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let root_dir = Arc::new(dirs::home_dir().unwrap());
    // let root_dir = Arc::new(PathBuf::from("/home/parthpant/dev"));
    let tera = Arc::new(Templates::new("templates/**/*.html"));

    let not_found = warp::get()
        .and(warp::any())
        .and(warp::path::full())
        .and(with_clone(&tera))
        .map(move |path: FullPath, tera: Arc<Templates>| {
            let mut context = Context::new();
            context.insert("path", path.as_str());
            let html = tera.render("404.html", &context).unwrap();
            warp::reply::html(html)
        });

    let folders = warp::get()
        .and(warp::path::full())
        .and(with_clone(&tera))
        .and(with_clone(&root_dir))
        .and_then(|reqpath: FullPath, tera: Arc<Templates>, home_dir: Arc<PathBuf>| async move {
            let reqpath = decode_url(reqpath.as_str().strip_prefix('/').unwrap());
            let reqpath = std::path::Path::new(&reqpath);
            let path = home_dir.join(&reqpath);
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

                    let mut context = Context::new();
                    context.insert("files", &files);
                    context.insert("dirs", &dirs);
                    context.insert("parent", &parent.to_str().unwrap());
                    context.insert("isroot", &(reqpath.to_str().unwrap() == ""));
                    context.insert("foldername", &path.file_name().unwrap().to_str().unwrap());

                    log::info!("reqpath: {:?}", reqpath);
                    log::info!("path: {:?}", path);
                    log::info!("parent: {:?}", parent);
                    log::info!("foldername: {:?}", path.file_name().unwrap());

                    let html = tera.render("list.html", &context).unwrap();
                    Ok(warp::reply::html(html))
                } else {
                    Err(warp::reject::not_found())
                }
            } else {
                Err(warp::reject::not_found())
            }
        });

    let files = warp::get()
        .and(warp::any())
        .and(warp::fs::dir("/home/parthpant/"));

    let routes = folders
        .or(files)
        .or(not_found)
        .with(warp::log("requests"));

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3000))
        .await;

    Ok(())
}

use clap::Parser;

/// Lightweight file server!
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path of the directory to be served
    #[arg(short, long, default_value_t = dirs::home_dir().unwrap().as_path().display().to_string())]
    pub dir: String,

    /// HTTP Port
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,
}

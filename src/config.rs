use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
#[clap(about = "Robocraft CRF archival system")]
pub struct CliArgs {
    /// Display more messages and more details
    #[clap(long)]
    pub verbose: bool,
    
    /// Path to SQLite database file to use
    #[clap(long)]
    pub database: Option<String>,
    
    /// Robots per page
    #[clap(short, long)]
    pub size: Option<isize>,

    /// Only look for new robots
    #[clap(short, long)]
    pub new: bool,

    /// Download known robots
    #[clap(short, long)]
    pub known: bool,

    /// Download thumbnails to this folder (default: don't download thumbnails)
    #[clap(short, long)]
    pub thumbnails: Option<std::path::PathBuf>,

    /// Re-download all thumbnails
    #[clap(long)]
    pub rethumb: bool,
}

pub fn parse() -> CliArgs {
    let args = CliArgs::parse();
    if args.rethumb && args.thumbnails.is_none() {
        eprintln!("Missing required thumbnails folder in CLI args");
        panic!("How do I re-download all thumbnails without a thumbnails folder to put them in?");
    }
    args
}

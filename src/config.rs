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
}

pub fn parse() -> CliArgs {
    CliArgs::parse()
}

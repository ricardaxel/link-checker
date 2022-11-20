use clap::Parser;

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    /// Directory in which the check will be done
    pub target_dir: String,
}

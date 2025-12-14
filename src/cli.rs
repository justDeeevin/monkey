use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg()]
    pub file: Option<PathBuf>,
    #[arg(long, default_value_t = Backend::Vm)]
    pub backend: Backend,
}

#[derive(clap::ValueEnum, strum::Display, Clone, Copy)]
#[strum(serialize_all = "kebab-case")]
pub enum Backend {
    Vm,
    Otf,
}

pub fn parse() -> Args {
    Args::parse()
}

use clap::Parser;
use color_eyre::eyre::Result;
use split_verilog_file::{SvDir, SvFile};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long)]
    output_dir: PathBuf,
}
fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let file_list = if args.input.is_file() {
        SvFile::new(args.input)
    } else if args.input.is_dir() {
        SvDir::build(args.input)
    } else {
        panic!("Input {:?} is neither a file or directory", args.input);
    };

    // Split the verilog files!
    split_verilog_file::process_files(file_list, args.output_dir);
    Ok(())
}

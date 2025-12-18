use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use sui_package_utils::graphql::PackageGraphQLFetcher;
use sui_package_utils::package_id_io::PackagesDir;
use sui_package_utils::package_saver::{save_package, SaveArgs};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    move_decompiler_path: PathBuf,
    #[arg(long)]
    packages_dir: PathBuf,
    #[arg(long)]
    initial_checkpoint: Option<u64>,
    #[arg(long)]
    max_checkpoint_seen_file: Option<PathBuf>,
    #[arg(long, default_value = "false")]
    force: bool,
}
fn main() -> Result<(), Box<dyn Error>> {
    let cli_args = Args::parse();
    let packages_dir = PackagesDir::new(cli_args.packages_dir.clone());
    let initial_checkpoint = if cli_args.initial_checkpoint.is_some() {
        cli_args.initial_checkpoint.unwrap()
    } else {
        packages_dir.get_latest_checkpoint()?
    };
    println!(
        "Fetching packages from graphql starting from checkpoint {}",
        initial_checkpoint
    );

    let mut fetcher = PackageGraphQLFetcher::new(initial_checkpoint, None);
    let res = fetcher.fetch_all()?;
    let new_max_checkpoint: u64 = if let Some(max) = res.iter().max_by_key(|pkg| pkg.checkpoint) {
        max.checkpoint
    } else {
        initial_checkpoint
    };
    println!(
        "{} new packages found. New max checkpoint seen: {}",
        res.len(),
        new_max_checkpoint
    );
    let save_args = SaveArgs {
        bcs: true,
        bytecode: true,
        call_graph: true,
        metadata: true,
        move_code: true,
        force: cli_args.force,
        packages_dir: packages_dir.get_prefix(),
        move_decompiler_path: cli_args.move_decompiler_path,
    };
    for pkg in res {
        save_package(&save_args, &pkg)?;
    }

    if let Some(max_checkpoint_seen_file) = cli_args.max_checkpoint_seen_file {
        let checkpoint_json = format!(
            "{{\n  \"max_checkpoint_seen\": \"{}\"\n}}",
            new_max_checkpoint
        );
        fs::write(max_checkpoint_seen_file, checkpoint_json)?;
    }
    Ok(())
}

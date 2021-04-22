use anyhow::{Context, Result};
use pico_args::Arguments;
use std::path::PathBuf;

mod codegen;

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcmd = args.subcommand()?.unwrap_or_default();

    goto_root()?;

    match subcmd.as_str() {
        "codegen" => {
            let protos = xshell::read_dir("crates/rpc/proto")?;

            let protos: Vec<_> = protos
                .iter()
                .filter(|f| f.is_file())
                .filter_map(|f| f.file_name())
                .filter_map(|f| f.to_str())
                .filter(|f| f.ends_with(".proto"))
                .map(|f| format!("proto/{}", f))
                .collect();

            let protos: Vec<_> = protos.iter().map(|s| s.as_str()).collect();

            codegen::codegen(&protos)?;
        }
        _ => eprintln!("cargo xtask codegen"),
    }

    Ok(())
}

fn goto_root() -> Result<()> {
    let git = PathBuf::from(".git");
    loop {
        if git.exists() {
            break Ok(());
        }
        let cwd = std::env::current_dir()?;
        let parent = cwd.parent().context("Could not find .git root")?;
        std::env::set_current_dir(parent)?;
    }
}

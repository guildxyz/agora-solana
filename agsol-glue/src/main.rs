use agsol_borsh_schema::{generate_layouts, generate_output};
use log::warn;
use structopt::StructOpt;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "-o", long, default_value = "contract-logic")]
    output: PathBuf,
    #[structopt(short = "-s", long)]
    schema: Option<PathBuf>,
    #[structopt(short = "-w", long)]
    wasm: Option<PathBuf>,
    #[structopt(short = "-t", long, default_value = "nodejs")]
    target: String,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let opt = Opt::from_args();

    if !opt.output.is_dir() {
        let mut cmd = Command::new("git")
            .args([
                "clone",
                "https://github.com/AgoraSpaceDAO/borsh-glue-template.git",
                &opt.output.to_string_lossy(),
            ])
            .spawn()?;

        cmd.wait()?;

        let mut cmd = Command::new("rm")
            .args(["-rf", &(opt.output.to_string_lossy() + "/.git")])
            .spawn()?;

        cmd.wait()?;
    }

    if let Some(schema_path) = opt.schema {
        let layouts = generate_layouts(schema_path)?;
        generate_output(&layouts, &opt.output)?;
    } else {
        warn!("No --schema <directory> provided. No schema was generated.");
    }

    if let Some(wasm_path) = opt.wasm {
        let mut path = std::env::current_dir()?;
        path.push(&opt.output);
        path.push("wasm-factory");
        let wasm_output_path = path.to_string_lossy().to_string();
        let mut cmd = Command::new("wasm-pack")
            .args([
                "build",
                &wasm_path.to_string_lossy(),
                "--out-dir",
                &wasm_output_path,
                "--out-name",
                "instructions",
                "--target",
                &opt.target,
            ])
            .spawn()?;

        cmd.wait()?;
    } else {
        warn!("[INFO] No --wasm <directory> provided. No wasm was generated.");
    }

    if opt.output.is_dir() {
        let mut file = fs::File::create(&*(opt.output.to_string_lossy() + "/.gitignore"))?;
        write!(file, "*")?;
    }

    Ok(())
}

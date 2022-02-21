use agsol_borsh_schema::{generate_layouts, generate_output};
use structopt::StructOpt;

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, StructOpt)]
enum GlueCmd {
    Schema {
        #[structopt(
            help = "path to the directory containing rust data structures for schema generation"
        )]
        path: PathBuf,
    },
    Wasm {
        #[structopt(help = "path to the directory containing wasm bindings")]
        path: PathBuf,
        #[structopt(short = "-t", long, help = "build target for 'wasm-pack'")]
        target: Option<String>,
        #[structopt(short = "-f", long, help = "extra features for 'wasm-pack'")]
        features: Option<Vec<String>>,
    },
}

#[derive(Debug, StructOpt)]
struct Glue {
    #[structopt(
        short = "-o",
        long,
        default_value = "glue",
        help = "output directory for the generated wasm and schema artifacts"
    )]
    output: PathBuf,
    #[structopt(subcommand)]
    cmd: GlueCmd,
}

fn main() -> Result<(), anyhow::Error> {
    let glue = Glue::from_args();

    clone_template(&glue.output)?;

    match glue.cmd {
        GlueCmd::Schema { path } => {
            let layouts = generate_layouts(path)?;
            generate_output(&layouts, &glue.output)?;
        }
        GlueCmd::Wasm {
            path,
            target,
            features,
        } => {
            let mut wasm_path = std::env::current_dir()?;
            wasm_path.push(&glue.output);
            wasm_path.push("wasm-bindings");
            let wasm_output_path = wasm_path.to_string_lossy().to_string();
            let mut args = vec![
                "build".to_owned(),
                path.to_string_lossy().to_string(),
                "--out-dir".to_owned(),
                wasm_output_path,
                "--out-name".to_owned(),
                "index".to_owned(),
            ];
            if let Some(target) = target {
                args.push("--target".to_owned());
                args.push(target);
            }

            if let Some(mut features) = features {
                args.push("--features".to_owned());
                args.append(&mut features);
            }

            let mut cmd = Command::new("wasm-pack").args(&args).spawn()?;
            cmd.wait()?;

            // remove auto-generated gitignore
            wasm_path.push(".gitignore");
            let mut cmd = Command::new("rm").args([wasm_path]).spawn()?;
            cmd.wait()?;
        }
    }

    Ok(())
}

fn clone_template(output_dir: &Path) -> Result<(), anyhow::Error> {
    if !output_dir.is_dir() {
        let output_dir_string = output_dir.to_string_lossy();
        let mut cmd = Command::new("git")
            .args([
                "clone",
                "https://github.com/AgoraSpaceDAO/borsh-glue-template.git",
                &output_dir_string,
            ])
            .spawn()?;

        cmd.wait()?;

        let mut cmd = Command::new("rm")
            .args(["-rf", &(output_dir.to_string_lossy() + "/.git")])
            .spawn()?;

        cmd.wait()?;
    }
    Ok(())
}

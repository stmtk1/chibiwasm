use anyhow::Result;
use chibiwasm::{execution::Runtime, wasi::WasiSnapshotPreview1};
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, about, version)]
struct Args {
    file: String,
    func: String,
    func_args: Option<Vec<i32>>,
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let Args {
        file,
        func,
        func_args,
    } = Args::parse();

    let args = match func_args {
        Some(args) => args.into_iter().map(Into::into).collect(),
        None => {
            vec![]
        }
    };

    let mut runtime = Runtime::from_file(&file, Some(Box::<WasiSnapshotPreview1>::default()))?;
    let result = runtime.call(func, args).expect("failed to call function");

    if let Some(output) = result {
        println!("{}", output);
    }
    Ok(())
}

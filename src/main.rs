use std::{cmp::Ordering, env::args_os, ffi::OsString, process::ExitCode};

use izanami::{run_file, run_prompt, RunError};

fn main() -> ExitCode {
    let args: Vec<OsString> = args_os().collect();

    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("usage: izanami [script]");
            return ExitCode::from(64);
        }
        Ordering::Equal => {
            let result = run_file(args[1].to_str().unwrap());

            if let Err(RunError::FileReadError(e)) = result {
                println!("Couldn't read the file. Reason: {}", e);
                return ExitCode::from(1);
            }
            if let Err(RunError::OtherError(e)) = result {
                println!("Error occured. Error: {}", e);
                return ExitCode::from(75);
            }

            if let Err(RunError::RuntimeError(_r)) = result {
                return ExitCode::from(70);
            }

            if let Err(RunError::ParseError) = result {
                return ExitCode::from(75);
            }
        }
        Ordering::Less => {
            let result = run_prompt();

            if let Err(res) = result {
                println!("Error while processing the repl. Reason: {}", &*res);
                return ExitCode::from(1);
            }
        }
    }

    ExitCode::SUCCESS
}

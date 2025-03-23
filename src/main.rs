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

            return match result {
                Err(RunError::FileReadError(e)) => {
                    println!("Couldn't read the file. Reason: {}", e);
                    return ExitCode::from(1);
                }
                Err(RunError::OtherError(e)) => {
                    println!("Error occured. Error: {}", e);
                    return ExitCode::from(75);
                }
                Err(RunError::RuntimeError(_)) => ExitCode::from(70),
                Err(RunError::ParseError) => ExitCode::from(75),
                Ok(_) => ExitCode::SUCCESS,
            };
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

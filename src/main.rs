use std::{env::args_os, ffi::OsString, process::ExitCode};

use izanami::{run_file, run_prompt};

fn main() -> ExitCode {
    let args: Vec<OsString> = args_os().collect();

    if args.len() > 2 {
        println!("usage: izanami [script]");
        return ExitCode::from(64);
    } else if args.len() == 2 {
        let result = run_file(args[1].to_str().unwrap());

        if let Err(res) = result {
            println!("Couldn't read the file. Reason: {}", &*res);
            return ExitCode::from(1);
        }
    } else {
        let result = run_prompt();

        if let Err(res) = result {
            println!("Error while processing the repl. Reason: {}", &*res);
            return ExitCode::from(1);
        }
    }

    ExitCode::SUCCESS
}

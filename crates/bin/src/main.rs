mod timed;

use std::process::ExitCode;

fn main() -> ExitCode {
    timed::run()
}

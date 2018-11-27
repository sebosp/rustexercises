extern crate steghide;
#[macro_use]
extern crate log;

use steghide::command;

fn main() {
    std::process::exit(match command::run_from_arguments() {
        Ok(()) => {
            info!("Successfully handled request. Exiting");
            0
        }
        Err(msg) => {
            error!("{}",msg);
            1
        }
    })
}

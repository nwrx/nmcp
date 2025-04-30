use structopt::StructOpt;
use unmcp::{Result, Program, ProgramArgs};

#[tokio::main]
async fn main() -> Result<()> {
    let args = ProgramArgs::from_args();
    Program::run(args).await
}

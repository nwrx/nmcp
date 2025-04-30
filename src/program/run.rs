use crate::{Program, Result};
use super::{program_struct::{Command, CrdCommand, OperatorCommand, SchemaCommand}, ProgramArgs};

impl Program {
    pub async fn run(args: ProgramArgs) -> Result<()> {

        // --- Initialize Pretty tracing
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(true)
            .with_line_number(false)
            .with_file(false)
            .without_time()
            // .pretty()
            .init();

        // --- Execute the command based on the provided arguments.
        match args.cmd {
            Command::Operator(OperatorCommand::Start(_)) => {
                let program = Program::from_args(args);
                program.operator_start().await
            },
            Command::Schema(schema_cmd) => {
                match schema_cmd {
                    SchemaCommand::Pool(format) => {
                        let output = Self::get_pool_schema(format.format).await?;
                        println!("{}", output);
                        Ok(())
                    },
                    SchemaCommand::Server(format) => {
                        let output = Self::get_server_schema(format.format).await?;
                        println!("{}", output);
                        Ok(())
                    }
                }
            },
            Command::Crd(crd_cmd) => {
                match crd_cmd {
                    CrdCommand::Pool(format) => {
                        let output = Self::get_pool_crd(format.format).await?;
                        println!("{}", output);
                        Ok(())
                    },
                    CrdCommand::Server(format) => {
                        let output = Self::get_server_crd(format.format).await?;
                        println!("{}", output);
                        Ok(())
                    }
                }
            }
        }
    }
}

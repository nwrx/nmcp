use crate::{Program, Result};

use super::{program_struct::{Command, CrdCommand, OperatorCommand, SchemaCommand}, ProgramArgs};

impl Program {
    /// Run the program and execute the appropriate command
    pub async fn run(args: ProgramArgs) -> Result<()> {
        match args.cmd {
            Command::Operator(OperatorCommand::Start(_)) => {
                let program = Program::from_args(args);
                program.oparator_start().await
            },
            Command::Schema(schema_cmd) => {
                match schema_cmd {
                    SchemaCommand::Pool(format_option) => {
                        Self::schema_pool(format_option.format).await
                    },
                    SchemaCommand::Server(format_option) => {
                        Self::schema_server(format_option.format).await
                    }
                }
            },
            Command::Crd(crd_cmd) => {
                match crd_cmd {
                    CrdCommand::Pool(format_option) => {
                        Self::crd_pool(format_option.format).await
                    },
                    CrdCommand::Server(format_option) => {
                        Self::crd_server(format_option.format).await
                    }
                }
            }
        }
    }
}
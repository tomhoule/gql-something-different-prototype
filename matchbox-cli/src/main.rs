extern crate matchbox_codegen;
#[macro_use]
extern crate structopt;

use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "matchbox", about = "The matchbox-graphql CLI")]
enum Command {
    /// Print the generated Rust code for a matchbox-graphql server from a GraphQL` schema
    #[structopt(name = "print-schema")]
    PrintSchema {
        /// The path to the GraphQL schema
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() {
    let command = Command::from_args();
    match command {
        Command::PrintSchema { file } => {
            let mut schema = String::new();
            ::std::fs::File::open(&file)
                .expect("the file can be opened")
                .read_to_string(&mut schema)
                .expect("the file is readable");
            println!("{}", matchbox_codegen::expand_schema(&schema));
        }
    }
}

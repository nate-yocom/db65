mod cpu;
mod debugger;
mod dis;
mod execute;
mod loader;
mod paravirt;
mod shell;

use anyhow::Result;
use std::path::PathBuf;

use crate::shell::Shell;
use clap::{Parser, Subcommand};
fn main() -> Result<()> {
    let cli = Cli::parse();
    // if  cli.set_exit
    let mut sh = Shell::new();
    sh.shell(cli.command_file)?;

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    command_file: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long)]
    set_exit: bool,
    // #[command(subcommand)]
    // command: Option<Commands>,
}

fn mainx() {
    // // You can check the value provided by positional arguments, or option arguments
    // if let Some(name) = cli.name.as_deref() {
    //     println!("Value for name: {name}");
    // }

    // if let Some(config_path) = cli.config.as_deref() {
    //     println!("Value for config: {}", config_path.display());
    // }

    // // You can see how many times a particular flag or argument occurred
    // // Note, only flags can have multiple occurrences
    // match cli.debug {
    //     0 => println!("Debug mode is off"),
    //     1 => println!("Debug mode is kind of on"),
    //     2 => println!("Debug mode is on"),
    //     _ => println!("Don't be crazy"),
    // }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    // match &cli.command {
    //     Some(Commands::Test { list }) => {
    //         if *list {
    //             println!("Printing testing lists...");
    //         } else {
    //             println!("Not printing testing lists...");
    //         }
    //     }
    //     None => {}
    // }

    // Continued program logic goes here...
}

mod io;
mod keys;
mod scanner;
mod decrypt;

use clap::{CommandFactory, Parser, Subcommand};
use keys::UserKeys;
use std::process::Command as ProcessCommand;

#[derive(Parser)]
#[command(name = "bcc-pack", version, about = "BCC Standalone Pack Utility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },
    Init,
    Decrypt {
        #[arg(value_name = "PACK | LIST | APK | DIR")]
        input: String,
    },
}

#[derive(Subcommand)]
enum KeysAction {
    Print,
    Load,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            UserKeys::prompt_interactive_load();
        }
        Some(Commands::Keys { action }) => match action {
            KeysAction::Print => {
                let keys = UserKeys::load();
                keys.print_status();
            }
            KeysAction::Load => {
                UserKeys::prompt_interactive_load();
            }
        },
        Some(Commands::Decrypt { input }) => {
            decrypt::execute(&input);
        }
        None => {
            let mut cmd = Cli::command();
            let _ = cmd.print_help();

            println!("\n--- Interactive Terminal Session Started ---");
            println!("Hint: Run the tool using ./bcc-pack <command>\n");

            if cfg!(target_os = "windows") {
                let _ = ProcessCommand::new("cmd.exe").status();
            } else {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
                let _ = ProcessCommand::new(shell).status();
            }
        }
    }
}
use std::{path::PathBuf, io::Write};
use std::fs::File;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a changelog entry
    Entry {
        /// Changelog entry type
        #[clap(arg_enum, value_parser)]
        entry_type: EntryType,

        /// Changelog entry message
        #[clap(value_parser)]
        message: String,

        /// Merge request number
        #[clap(short, long, value_parser)]
        mr_number: Option<u32>,
        
        /// Realted issue number
        #[clap(short, long, value_parser)]
        issue_number: Option<u32>,

        /// Path to the directory with changelog entries
        #[clap(value_parser)]
        entry_dir_path: PathBuf,
    },

    /// Generates changelog
    Generate,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum EntryType {
    Added,
    Fixed,
    Removed,
}

fn generate_changelog_entry_file_name(message: &String) -> String {
    let mut entry_file_name = message.clone();
    entry_file_name.retain(|c| !c.is_whitespace());
    entry_file_name.push_str(".txt");

    entry_file_name
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Entry {entry_type, message, mr_number, issue_number, entry_dir_path}) => {
            let mut entry_str: String = match entry_type {
                EntryType::Added => {String::from("Added: ")}
                EntryType::Fixed => {String::from("Fixed: ")}
                EntryType::Removed => {String::from("Removed: ")}
            };
            
            entry_str.push_str(message);

            if let Some(mr_n) = mr_number {
                entry_str.push_str(&format!(" [MR ${mr_n}](https://link/to/{mr_n}): {mr_n}", mr_n=mr_n));
            }

            if let Some(issue_n) = issue_number {
                entry_str.push_str(&format!(" [Issue #{issue_n}](https://link/to/{issue_n})", issue_n=issue_n));
            }
            println!("{}", entry_str);

            let entry_file_name = generate_changelog_entry_file_name(message);            
            let mut entry_file_path = entry_dir_path.clone();
            entry_file_path.push(entry_file_name);
            if entry_file_path.exists() {
                panic!("File with the same entry already exists.");
            }
            let mut output = File::create(entry_file_path)?;
            output.write_all(entry_str.as_bytes())?;
        }
        Some(Commands::Generate) => {
            panic!("Changelog generation has not been supported yet!");
        }
        None => {}
    }

    Ok(())
}

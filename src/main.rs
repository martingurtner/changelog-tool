use std::fs::File;
use std::{io::Write, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
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
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

struct Entry<'a> {
    entry_type: &'a EntryType,
    message: &'a String,
    issue_number: &'a Option<u32>,
    mr_number: &'a Option<u32>,
}

fn generate_changelog_entry_string(entry: &Entry) -> String {
    let mut entry_str = String::with_capacity(500);

    match entry.entry_type {
        EntryType::Added =>  entry_str.push_str("Added: "),
        EntryType::Changed =>  entry_str.push_str("Changed: "),
        EntryType::Deprecated =>  entry_str.push_str("Deprecated: "),
        EntryType::Removed =>  entry_str.push_str("Removed: "),
        EntryType::Fixed =>  entry_str.push_str("Fixed: "),
        EntryType::Security =>  entry_str.push_str("Security: ")
    };

    entry_str.push_str(entry.message);

    if let Some(mr_n) = entry.mr_number {
        entry_str.push_str(&format!(
            " [[MR-!{mr_n}](http://rheldispptapp1/gitlab/kcvc/kcvc/-/merge_requests/{mr_n})]",
            mr_n = mr_n
        ));
    }

    if let Some(issue_n) = entry.issue_number {
        entry_str.push_str(&format!(
            " [[ISSUE-#{issue_n}](http://rheldispptapp1/gitlab/kcvc/kcvc/-/merge_requests/{issue_n})]",
            issue_n = issue_n
        ));
    }

    entry_str
}

fn generate_changelog_entry_file_name(entry: &Entry) -> String {
    let mut entry_file_name = String::with_capacity(100);

    if let Some(issue_n) = entry.issue_number {
        entry_file_name.push_str("IN");
        entry_file_name.push_str(&issue_n.to_string());
        entry_file_name.push('_');
    }

    if let Some(mr_n) = entry.mr_number {
        entry_file_name.push_str("MR");
        entry_file_name.push_str(&mr_n.to_string());
        entry_file_name.push('_');
    }

    for c in entry.message.chars() {
        if c.is_whitespace() {
            entry_file_name.push('_');
        } else {
            entry_file_name.push(c);
        }
    }
    entry_file_name.push_str(".md");

    entry_file_name
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Entry {
            entry_type,
            message,
            mr_number,
            issue_number,
            entry_dir_path,
        } => {
            let entry = Entry {
                entry_type,
                message,
                mr_number,
                issue_number,
            };

            let entry_str = generate_changelog_entry_string(&entry);

            let entry_file_name = generate_changelog_entry_file_name(&entry);
            let mut entry_file_path = entry_dir_path.clone();
            entry_file_path.push(&entry_file_name);
            if entry_file_path.exists() {
                panic!("File with the same entry already exists.");
            }
            let mut output = File::create(entry_file_path)?;
            output.write_all(entry_str.as_bytes())?;

            println!(
                "Message: \n\t\"{}\"\nwas written to file \"{}\"",
                entry_str, entry_file_name
            );
        }
        Commands::Generate => {
            panic!("Changelog generation has not been supported yet!");
        }
    }

    Ok(())
}

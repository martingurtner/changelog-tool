use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::{io::Write, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

static ISSUE_URL: &str = "http://gitlab/repo/-/issues/";
static MERGE_REQUEST_URL: &str = "http://gitlab/repo/-/merge_requests/";

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
    Generate {
        /// Path to the directory with changelog entries
        #[clap(value_parser)]
        entry_dir_path: PathBuf,
    },
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

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryType::Added => write!(f, "Added"),
            EntryType::Changed => write!(f, "Changed"),
            EntryType::Deprecated => write!(f, "Deprecated"),
            EntryType::Removed => write!(f, "Removed"),
            EntryType::Fixed => write!(f, "Fixed"),
            EntryType::Security => write!(f, "Security"),
        }
    }
}

struct Entry<'a> {
    entry_type: &'a EntryType,
    message: &'a String,
    issue_number: &'a Option<u32>,
    mr_number: &'a Option<u32>,
}

impl fmt::Display for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result: fmt::Result;
        result = write!(f, "{}: {}", self.entry_type, self.message);

        if let Some(mr_n) = self.mr_number {
            result = write!(
                f,
                " [[MR-!{mr_n}]({mergee_request_url}/{mr_n})]",
                mr_n = mr_n,
                mergee_request_url = MERGE_REQUEST_URL,
            );
        }

        if let Some(issue_n) = self.issue_number {
            result = write!(
                f,
                " [[ISSUE-#{issue_n}]({issue_url}{issue_n})]",
                issue_n = issue_n,
                issue_url = ISSUE_URL,
            );
        }

        result
    }
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

            let entry_str = entry.to_string();

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
        Commands::Generate { entry_dir_path } => {
            let mut changelog_entries: HashMap<std::string::String, Vec<std::string::String>> =
                HashMap::new();

            let paths = fs::read_dir(entry_dir_path).unwrap();
            let md_files = paths
                .into_iter()
                .filter(|p| p.as_ref().unwrap().path().extension().unwrap() == "md");

            for path in md_files {
                let p_path = path.unwrap().path();
                let file_content = fs::read_to_string(&p_path).unwrap();

                match file_content.split_once(": ") {
                    Some((entry_type, entry_msg)) => {
                        let entry_type_str = String::from(entry_type);
                        let entry_msg_str = String::from(entry_msg);

                        changelog_entries
                            .entry(entry_type_str)
                            .and_modify(|values| values.push(entry_msg_str.clone()))
                            .or_insert(vec![entry_msg_str]);
                    }
                    None => println!(
                        "Skipping {} due to a wrong format of the message.",
                        p_path.display()
                    ),
                }
            }

            println!("# Changelog:");
            for (k, v) in changelog_entries {
                println!("## {k}: ");
                for entry in v {
                    println!(" * {entry}");
                }
            }
        }
    }

    Ok(())
}

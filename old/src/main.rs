use clap::Parser;

use std::env;
use std::path::{Path, PathBuf};
use clap::{ArgEnum, Subcommand};
use std::process::{Child, Command, Stdio};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    repos: Vec<Repo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Repo {
    url: String,
    dest: String,
}



#[derive(Parser,Debug)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand,Debug)]
pub enum Action {
    Sync,
}

impl Action {
    pub fn handle(self) {
        use Action::*;

        // routing.
        match self {
            Sync => sync_command(),
        }
    }
}

use git2::Repository;
use std::fs;

pub fn sync_command() {
    let content = fs::read_to_string("upkg.yaml").unwrap();
    println!("yaml {content}");
    let config: Config = serde_yaml::from_str(&content).unwrap();

    for repo in config.repos {
        println!("Cloning {} into {}", repo.url, repo.dest);
        Repository::clone(&repo.url, repo.dest).unwrap();
    }
}

pub fn run() {
    Cli::parse().action.handle();
}

fn main() {
    println!("Hello, world!");
    run();
}

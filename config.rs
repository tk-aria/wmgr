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


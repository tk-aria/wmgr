use git2::Repository;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = build_cli().get_matches();

    if let Some(_) = matches.subcommand_matches("sync") {
        let content = fs::read_to_string("upkg.yaml")?;
        let config: Config = serde_yaml::from_str(&content)?;

        for repo in config.repos {
            println!("Cloning {} into {}", repo.url, repo.dest);
            Repository::clone(&repo.url, repo.dest)?;
        }
    }

    Ok(())
}



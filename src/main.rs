use anyhow::Context;
use clap::{Parser, Subcommand};
use dialoguer::{FuzzySelect, Input};
use serde::{Deserialize, Serialize};
use slug::slugify;
use std::{
    collections::HashMap,
    env::{self, current_dir},
    fmt::Display,
    fs,
    path::PathBuf,
    process::Command,
};

#[derive(Serialize, Deserialize)]
struct Config {
    repos: HashMap<String, Repo>,
}

#[derive(Serialize, Deserialize)]
struct Repo {
    base_directory: PathBuf,
    all_workspaces: PathBuf,
    workspaces: HashMap<String, Workspace>,
    workspace_init_command: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Workspace {
    directory: PathBuf,
    tab_id: Option<u32>,
    description: String,
}

fn home_dir() -> anyhow::Result<PathBuf> {
    env::home_dir().context("Failed to get home directory")
}

fn bleebo_dir() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".bleebo-code"))
}

fn config_path() -> anyhow::Result<PathBuf> {
    Ok(bleebo_dir()?.join("config.toml"))
}

fn workspaces_path() -> anyhow::Result<PathBuf> {
    Ok(bleebo_dir()?.join("repos"))
}

fn get_or_create_config() -> anyhow::Result<Config> {
    if !fs::exists(config_path()?)? {
        fs::create_dir_all(bleebo_dir()?)?;
        let config = Config {
            repos: HashMap::new(),
        };
        put_config(config)?;
        println!("Created config at ~/.bleebo-code/config.toml");
    }
    let config: Config = toml::from_str(&fs::read_to_string(config_path()?)?)?;
    Ok(config)
}

fn put_config(config: Config) -> anyhow::Result<()> {
    fs::write(config_path()?, toml::to_string_pretty(&config)?)?;
    Ok(())
}

// TODO: this should probably be a method on Config
fn get_current_repo_slug(config: &Config) -> anyhow::Result<String> {
    let cwd = current_dir()?;
    for (repo_slug, repo) in config.repos.iter() {
        if cwd.starts_with(repo.base_directory.clone()) {
            return Ok(repo_slug.clone());
        }
        for workspace in repo.workspaces.values() {
            if cwd.starts_with(workspace.directory.clone()) {
                return Ok(repo_slug.clone());
            }
        }
    }
    anyhow::bail!("Not currently in a repo");
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize bleebo-code for a repo
    Init {
        /// Name of the repo
        name: String,
    },

    /// Create a new workspace in the current repo
    New {
        /// Parent change to create the workspace on. Defaults to `trunk()`
        parent: Option<String>,
    },

    /// Select a workspace in the current repo to switch into
    List,

    /// Open the config file using $EDITOR
    Config,
}

fn command_init(mut config: Config, name: &String) -> anyhow::Result<()> {
    let slug = slugify(name);
    let all_workspaces: PathBuf = workspaces_path()?.join(&slug);
    fs::create_dir_all(&all_workspaces).context(format!(
        "Failed to create workspace directory at {all_workspaces:#?}"
    ))?;
    let repo = Repo {
        base_directory: current_dir()?,
        all_workspaces,
        workspaces: HashMap::new(),
        workspace_init_command: None,
    };
    config.repos.insert(slug, repo);
    put_config(config)?;
    Ok(())
}

fn command_new(mut config: Config, parent: Option<&str>) -> anyhow::Result<()> {
    let repo_slug = get_current_repo_slug(&config)?;
    let repo = config
        .repos
        .get_mut(&repo_slug)
        .context("Could not get repo from slug {repo_slug}")?;
    let description: String = Input::new()
        .with_prompt("pls describe new workspace")
        .interact_text()?;
    let workspace_slug = slugify(description.clone());
    let workspace_directory: PathBuf = repo
        .all_workspaces
        .join(PathBuf::from(workspace_slug.clone()));
    let workspace_directory_str = workspace_directory
        .to_str()
        .context("Failed to convert workspace directory to str")?;
    let parent = parent.unwrap_or("trunk()");
    let stderr = Command::new("jj")
        .args([
            "workspace",
            "add",
            workspace_directory_str,
            "-r",
            parent,
            "-m",
            &description,
        ])
        .output()?
        .stderr;
    print!("{}", String::from_utf8(stderr.clone())?);

    let output = Command::new("zellij")
        .args([
            "action",
            "new-tab",
            "--cwd",
            workspace_directory_str,
            "--layout-string",
            include_str!("../default-layout.kdl"),
            "--name",
            &description,
        ])
        .output()?;
    print!("{}", String::from_utf8(output.stderr.clone())?);

    let tab_id: u32 = String::from_utf8(output.stdout)?.trim().parse()?;

    let output = Command::new("zellij")
        .args([
            "action",
            "rename-tab",
            "--tab-id",
            &tab_id.to_string(),
            &description,
        ])
        .output()?;
    print!("{}", String::from_utf8(output.stderr.clone())?);

    if let Some(init_command) = repo.workspace_init_command.clone() {
        let output = Command::new("sh")
            .current_dir(&workspace_directory)
            .args(["-c", &init_command])
            .output()?;
        print!("{}", String::from_utf8(output.stderr.clone())?);
    }

    let workspace = Workspace {
        directory: workspace_directory,
        tab_id: Some(tab_id),
        description,
    };

    repo.workspaces.insert(workspace_slug, workspace);
    put_config(config)?;

    Ok(())
}

impl Display for Workspace {
    // TODO: we might only want this for command_list
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.description,
            self.tab_id
                .map(|x| format!("tab {x}"))
                .unwrap_or("no tab".to_string())
        )
    }
}

fn command_list(config: Config) -> anyhow::Result<()> {
    let repo_slug = get_current_repo_slug(&config)?;
    let repo = config
        .repos
        .get(&repo_slug)
        .context("Could not get repo from slug {repo_slug}")?;
    let workspaces: Vec<&Workspace> = repo.workspaces.values().collect();
    let selection = FuzzySelect::new()
        .with_prompt("select a workspace")
        .items(workspaces.iter())
        .interact()?;
    let workspace = workspaces[selection];
    let output = Command::new("zellij")
        .args([
            "action",
            "go-to-tab-by-id",
            &workspace
                .tab_id
                .context("workspace has no tab")?
                .to_string(),
        ])
        .output()?;
    print!("{}", String::from_utf8(output.stderr.clone())?);
    Ok(())
}

pub fn command_config() -> anyhow::Result<()> {
    let config_path = config_path()?;
    println!("{}", config_path.display());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let config = get_or_create_config().context("Failed to read config")?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => command_init(config, name),
        Commands::New { parent } => command_new(config, parent.as_deref()),
        Commands::List => command_list(config),
        Commands::Config => command_config(),
    }
}

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct Jujutsu {
    repo_path: String,
}

impl Jujutsu {
    pub fn new(repo_path: impl Into<String>) -> Self {
        Jujutsu {
            repo_path: repo_path.into(),
        }
    }

    /// Initialize a new Jujutsu repository
    pub fn init(&self) -> Result<()> {
        Command::new("jj")
            .arg("init")
            .arg(&self.repo_path)
            .output()
            .context("Failed to initialize Jujutsu repo")?;
        Ok(())
    }

    /// Check if repo exists
    pub fn repo_exists(&self) -> bool {
        Path::new(&self.repo_path).join(".jj").exists()
    }

    /// Create a new commit (note) with the given content
    pub fn create_commit(&self, message: &str, content: &str) -> Result<String> {
        // Write content to a temporary file
        let file_path = format!("{}/notes/{}.md", self.repo_path, message.replace(" ", "_"));
        
        // Ensure notes directory exists
        std::fs::create_dir_all(format!("{}/notes", self.repo_path))
            .context("Failed to create notes directory")?;
        
        // Write note content
        std::fs::write(&file_path, content)
            .context("Failed to write note file")?;

        // Add file to jj
        Command::new("jj")
            .arg("new")
            .arg("-m")
            .arg(message)
            .arg(&file_path)
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create commit")?;

        // Get the commit hash
        let output = Command::new("jj")
            .arg("log")
            .arg("-r")
            .arg("@")
            .arg("--no-graph")
            .arg("--template")
            .arg("{commit_id}")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get commit hash")?;

        let commit_id = String::from_utf8(output.stdout)
            .context("Failed to parse commit ID")?
            .trim()
            .to_string();

        Ok(commit_id)
    }

    /// Get commit content by commit ID
    pub fn get_commit_content(&self, _commit_id: &str) -> Result<String> {
        // This is simplified - in reality, you'd need to reconstruct the note
        // from the commit history. For now, we'll store notes separately.
        Ok(String::new())
    }

    /// List all commits (notes)
    pub fn list_commits(&self) -> Result<Vec<String>> {
        let output = Command::new("jj")
            .arg("log")
            .arg("--no-graph")
            .arg("--template")
            .arg("{commit_id} {description}\n")
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to list commits")?;

        let output_str = String::from_utf8(output.stdout)
            .context("Failed to parse commit list")?;

        let commits: Vec<String> = output_str
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(commits)
    }
}


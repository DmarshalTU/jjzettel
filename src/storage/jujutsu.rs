use anyhow::{Context, Result};
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

    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    /// Initialize a new Jujutsu repository
    pub fn init(&self) -> Result<()> {
        let repo_path_buf = std::path::Path::new(&self.repo_path);
        let repo_path_abs = if repo_path_buf.is_absolute() {
            match repo_path_buf.canonicalize() {
                Ok(path) => path,
                Err(_) => {
                    std::fs::create_dir_all(repo_path_buf)?;
                    repo_path_buf.to_path_buf()
                }
            }
        } else {
            let cwd = std::env::current_dir()?;
            let path = cwd.join(repo_path_buf);
            match path.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    std::fs::create_dir_all(&path)?;
                    path
                }
            }
        };
        
        let output = Command::new("jj")
            .arg("git")
            .arg("init")
            .arg(&repo_path_abs)
            .output()
            .context("Failed to initialize Jujutsu repo")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to initialize repo: {}", stderr);
        }
        
        Ok(())
    }

    /// Check if repo exists
    pub fn repo_exists(&self) -> bool {
        let repo_path_buf = std::path::Path::new(&self.repo_path);
        let repo_path_abs = if repo_path_buf.is_absolute() {
            repo_path_buf.to_path_buf()
        } else {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| Some(cwd.join(repo_path_buf)))
                .unwrap_or(repo_path_buf.to_path_buf())
        };
        repo_path_abs.join(".jj").exists()
    }

    /// Create a new commit for a file
    pub fn create_commit_for_file(&self, message: &str, file_path: &str) -> Result<String> {
        // Ensure the file exists (should already be written by caller)
        if !std::path::Path::new(file_path).exists() {
            anyhow::bail!("File does not exist: {}", file_path);
        }

        // Ensure repo path is absolute
        let repo_path_buf = std::path::Path::new(&self.repo_path);
        let repo_path_abs = if repo_path_buf.is_absolute() {
            repo_path_buf.canonicalize()
                .context("Failed to canonicalize repo path")?
        } else {
            std::env::current_dir()?
                .join(repo_path_buf)
                .canonicalize()
                .context("Failed to canonicalize repo path")?
        };
        
        // Ensure repo is initialized
        if !repo_path_abs.join(".jj").exists() {
            std::fs::create_dir_all(&repo_path_abs)?;
            let output = Command::new("jj")
                .arg("git")
                .arg("init")
                .arg(&repo_path_abs)
                .output()
                .context("Failed to initialize Jujutsu repo")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to initialize repo: {}", stderr);
            }
        }

        // Create a new commit with the file
        // For new files, we create a commit from the working copy
        // Jujutsu will automatically include all changes in the working copy
        let output = Command::new("jj")
            .arg("new")
            .arg("-m")
            .arg(message)
            .current_dir(&repo_path_abs)
            .output()
            .context("Failed to create commit")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create commit: {}", stderr);
        }

        let output = Command::new("jj")
            .arg("log")
            .arg("-r")
            .arg("@")
            .arg("--no-graph")
            .arg("--template")
            .arg("{commit_id}")
            .current_dir(&repo_path_abs)
            .output()
            .context("Failed to get commit hash")?;

        let commit_id = String::from_utf8(output.stdout)
            .context("Failed to parse commit ID")?
            .trim()
            .to_string();

        Ok(commit_id)
    }

    /// Get commit history for a specific file with optional title filtering
    pub fn get_file_history_with_title(&self, file_path: &str, note_title: &str) -> Result<Vec<CommitInfo>> {
        // Ensure repo path is absolute
        let repo_path_buf = std::path::Path::new(&self.repo_path);
        let repo_path_abs = if repo_path_buf.is_absolute() {
            repo_path_buf.canonicalize()
                .context("Failed to canonicalize repo path")?
        } else {
            std::env::current_dir()?
                .join(repo_path_buf)
                .canonicalize()
                .context("Failed to canonicalize repo path")?
        };
        
        // Check if repo exists
        if !repo_path_abs.join(".jj").exists() {
            return Ok(Vec::new());
        }
        
        let file_path_buf = std::path::Path::new(file_path);
        let file_path_abs = if file_path_buf.is_absolute() {
            file_path_buf.canonicalize()
                .context("Failed to canonicalize file path")?
        } else {
            std::env::current_dir()?
                .join(file_path_buf)
                .canonicalize()
                .context("Failed to canonicalize file path")?
        };
        
        let relative_path = file_path_abs.strip_prefix(&repo_path_abs)
            .context("File is not in repo")?
            .to_string_lossy()
            .to_string();

        // Try to get history for the specific file first
        let file_output = Command::new("jj")
            .arg("log")
            .arg("--no-graph")
            .arg("-T")
            .arg(r#"commit_id.short() ++ " | " ++ if(description == "", "(empty)", description) ++ " | " ++ author.name()"#)
            .arg(&relative_path)
            .current_dir(&repo_path_abs)
            .output();
        
        // Get all commits as fallback
        let all_output = Command::new("jj")
            .arg("log")
            .arg("--no-graph")
            .arg("-T")
            .arg(r#"commit_id.short() ++ " | " ++ if(description == "", "(empty)", description) ++ " | " ++ author.name()"#)
            .current_dir(&repo_path_abs)
            .output()
            .context("Failed to get commit history")?;
        
        let all_output_str = String::from_utf8(all_output.stdout)
            .context("Failed to parse commit history")?;

        let mut commits = Vec::new();
        
        // If file-specific lookup worked, use that (but still filter by note title)
        if let Ok(output) = file_output {
            if output.status.success() {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    // Normalize the output - handle lines that start with " | " or "| "
                    // Jujutsu sometimes wraps output with continuation lines starting with " | "
                    let normalized = output_str
                        .lines()
                        .map(|line| {
                            let trimmed = line.trim();
                            if trimmed.starts_with("| ") {
                                // This is a continuation line, remove the leading "| "
                                trimmed.strip_prefix("| ").unwrap_or(trimmed).to_string()
                            } else if trimmed.starts_with(" | ") {
                                // Handle lines starting with " | " (space-pipe-space)
                                trimmed.strip_prefix(" | ").unwrap_or(trimmed).to_string()
                            } else {
                                trimmed.to_string()
                            }
                        })
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<String>>()
                        .join("\n");
                    
                    for line in normalized.lines() {
                        if line.is_empty() || line.trim().is_empty() {
                            continue;
                        }
                        let parts: Vec<&str> = line.split(" | ").collect();
                        if parts.len() >= 2 {
                            let id = parts[0].trim();
                            let message = parts[1].trim();
                            let author = if parts.len() >= 3 {
                                parts[2].trim()
                            } else {
                                ""
                            };
                            
                            // Filter by note title if provided (case-insensitive)
                            let should_include = if message == "(empty)" {
                                false
                            } else if !note_title.is_empty() {
                                let message_lower = message.to_lowercase();
                                let title_lower = note_title.to_lowercase();
                                message_lower.contains(&title_lower)
                            } else {
                                true
                            };
                            
                            if should_include && !id.is_empty() {
                                commits.push(CommitInfo {
                                    id: id.to_string(),
                                    message: message.to_string(),
                                    author: author.to_string(),
                                    timestamp: if parts.len() >= 4 {
                                        parts[3].trim().to_string()
                                    } else {
                                        String::new()
                                    },
                                });
                            }
                        }
                    }
                    if !commits.is_empty() {
                        return Ok(commits);
                    }
                }
            }
        }
        
        // Otherwise, filter all commits by checking if they mention this file
        // or if the commit message matches note patterns
        // Since commits are created from working copy, we need to check all commits
        // and filter by commit message patterns
        for line in all_output_str.lines() {
            if line.is_empty() || line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() >= 2 {
                let id = parts[0].trim();
                let message = parts[1].trim();
                let author = if parts.len() >= 3 {
                    parts[2].trim()
                } else {
                    ""
                };
                
                // Include commits that match this specific note
                // Check if the commit message contains the note title
                // Commit messages are like "Note: {title}", "Update: {title}", etc.
                // Include commits that match this specific note title
                // Exclude "(empty)" commits
                let should_include = if message == "(empty)" {
                    false
                } else if !note_title.is_empty() {
                    // If we have a note title, match commits that contain it (case-insensitive)
                    let message_lower = message.to_lowercase();
                    let title_lower = note_title.to_lowercase();
                    message_lower.contains(&title_lower)
                } else {
                    // Fallback: include commits with note-related prefixes
                    message.contains("Note:")
                        || message.contains("Update:")
                        || message.contains("Duplicate:")
                };
                
                if should_include && !id.is_empty() {
                    commits.push(CommitInfo {
                        id: id.to_string(),
                        message: message.to_string(),
                        author: author.to_string(),
                        timestamp: if parts.len() >= 4 {
                            parts[3].trim().to_string()
                        } else {
                            String::new()
                        },
                    });
                }
            }
        }
        
        // Jujutsu returns commits in chronological order (oldest first)
        // Reverse to show newest first
        commits.reverse();


        Ok(commits)
    }

    #[allow(dead_code)]
    pub fn get_file_history(&self, file_path: &str) -> Result<Vec<CommitInfo>> {
        self.get_file_history_with_title(file_path, "")
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}


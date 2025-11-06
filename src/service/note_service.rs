use anyhow::Result;
use crate::storage::jujutsu::Jujutsu;
use crate::storage::note::Note;
use std::path::PathBuf;
use std::process::Command;

pub struct NoteService {
    jujutsu: Jujutsu,
    notes_dir: PathBuf,
}

impl NoteService {
    pub fn new(repo_path: impl Into<String>) -> Self {
        let repo_path_str = repo_path.into();
        let notes_dir = PathBuf::from(&repo_path_str).join("notes");
        
        NoteService {
            jujutsu: Jujutsu::new(&repo_path_str),
            notes_dir,
        }
    }

    /// Initialize the service (create repo if needed)
    pub fn initialize(&self) -> Result<()> {
        if !self.jujutsu.repo_exists() {
            self.jujutsu.init()?;
        }
        
        // Ensure notes directory exists
        std::fs::create_dir_all(&self.notes_dir)?;
        
        Ok(())
    }

    /// Create a new note
    pub fn create_note(&self, title: String, content: String) -> Result<Note> {
        let note = Note::new(title.clone(), content.clone());
        
        // Save note to file first
        let note_file = self.notes_dir.join(format!("{}.json", note.id));
        let note_json = serde_json::to_string_pretty(&note)?;
        std::fs::write(&note_file, note_json)?;
        
        // Create commit in Jujutsu for the actual JSON file
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let commit_message = format!("Note: {} ({})", title, timestamp);
        let file_path_str = note_file.to_string_lossy().to_string();
        let _commit_id = self.jujutsu.create_commit_for_file(&commit_message, &file_path_str)?;
        
        Ok(note)
    }

    /// Duplicate a note (creates a copy with a new ID)
    pub fn duplicate_note(&self, note_id: &str) -> Result<Note> {
        let original_note = self.get_note(note_id)?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;
        
        // Create new note with "Copy of" prefix
        let new_title = format!("Copy of {}", original_note.title);
        let new_note = Note::new(new_title.clone(), original_note.content.clone());
        
        // Copy tags but not links (user can link manually)
        let mut new_note_with_tags = new_note;
        new_note_with_tags.tags = original_note.tags.clone();
        
        // Save duplicated note
        let note_file = self.notes_dir.join(format!("{}.json", new_note_with_tags.id));
        let note_json = serde_json::to_string_pretty(&new_note_with_tags)?;
        std::fs::write(&note_file, note_json)?;
        
        // Create commit in Jujutsu for the actual JSON file
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let commit_message = format!("Duplicate: {} ({})", new_title, timestamp);
        let file_path_str = note_file.to_string_lossy().to_string();
        let _commit_id = self.jujutsu.create_commit_for_file(&commit_message, &file_path_str)?;
        
        Ok(new_note_with_tags)
    }

    /// Load all notes
    pub fn list_notes(&self) -> Result<Vec<Note>> {
        let mut notes = Vec::new();
        
        if !self.notes_dir.exists() {
            return Ok(notes);
        }
        
        for entry in std::fs::read_dir(&self.notes_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)?;
                let note: Note = serde_json::from_str(&content)?;
                notes.push(note);
            }
        }
        
        // Sort by updated_at (most recent first)
        notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(notes)
    }

    /// Get a note by ID
    pub fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let note_file = self.notes_dir.join(format!("{}.json", id));
        
        if !note_file.exists() {
            return Ok(None);
        }
        
        let content = std::fs::read_to_string(&note_file)?;
        let note: Note = serde_json::from_str(&content)?;
        Ok(Some(note))
    }

    /// Get all notes that link to the given note (backlinks)
    pub fn get_backlinks(&self, note_id: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_notes()?;
        
        let backlinks: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| note.links.contains(&note_id.to_string()))
            .collect();
        
        Ok(backlinks)
    }

    /// Update a note
    pub fn update_note(&self, mut note: Note, new_content: String) -> Result<Note> {
        note.content = new_content;
        note.updated_at = chrono::Utc::now().to_rfc3339();
        
        // Save updated note
        let note_file = self.notes_dir.join(format!("{}.json", note.id));
        let note_json = serde_json::to_string_pretty(&note)?;
        std::fs::write(&note_file, note_json)?;
        
        // Create commit in Jujutsu for the actual JSON file
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let commit_message = format!("Update: {} ({})", note.title, timestamp);
        let file_path_str = note_file.to_string_lossy().to_string();
        self.jujutsu.create_commit_for_file(&commit_message, &file_path_str)?;
        
        Ok(note)
    }

    /// Add a tag to a note
    pub fn add_tag(&self, note_id: &str, tag: String) -> Result<Note> {
        let mut note = self.get_note(note_id)?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;
        
        let tag_lower = tag.to_lowercase();
        if !note.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
            note.tags.push(tag);
            note.updated_at = chrono::Utc::now().to_rfc3339();
            
            // Save updated note
            let note_file = self.notes_dir.join(format!("{}.json", note.id));
            let note_json = serde_json::to_string_pretty(&note)?;
            std::fs::write(&note_file, note_json)?;
        }
        
        Ok(note)
    }

    /// Remove a tag from a note
    pub fn remove_tag(&self, note_id: &str, tag: &str) -> Result<Note> {
        let mut note = self.get_note(note_id)?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;
        
        let tag_lower = tag.to_lowercase();
        note.tags.retain(|t| t.to_lowercase() != tag_lower);
        note.updated_at = chrono::Utc::now().to_rfc3339();
        
        // Save updated note
        let note_file = self.notes_dir.join(format!("{}.json", note.id));
        let note_json = serde_json::to_string_pretty(&note)?;
        std::fs::write(&note_file, note_json)?;
        
        Ok(note)
    }

    /// Search notes by tags
    pub fn search_by_tag(&self, tag: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_notes()?;
        let tag_lower = tag.to_lowercase();
        
        let filtered: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| {
                note.tags.iter().any(|t| t.to_lowercase() == tag_lower)
            })
            .collect();
        
        Ok(filtered)
    }

    /// Delete a note
    pub fn delete_note(&self, id: &str) -> Result<()> {
        let note_file = self.notes_dir.join(format!("{}.json", id));
        
        if note_file.exists() {
            // Delete the file
            std::fs::remove_file(&note_file)?;
            
            // Create commit in Jujutsu for deletion
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let commit_message = format!("Delete note: {} ({})", id, timestamp);
            // Just describe the deletion - the file is already removed
            Command::new("jj")
                .arg("describe")
                .arg("-m")
                .arg(&commit_message)
                .current_dir(&self.jujutsu.repo_path())
                .output()?;
        }
        
        Ok(())
    }

    /// Search notes by title or content, or by tag if query starts with #
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>> {
        let all_notes = self.list_notes()?;
        
        // If query starts with #, search by tag
        if query.starts_with('#') {
            let tag = query.trim_start_matches('#').trim();
            if tag.is_empty() {
                return Ok(all_notes);
            }
            return self.search_by_tag(tag);
        }
        
        // Otherwise search by title or content
        let query_lower = query.to_lowercase();
        
        let filtered: Vec<Note> = all_notes
            .into_iter()
            .filter(|note| {
                note.title.to_lowercase().contains(&query_lower) ||
                note.content.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(filtered)
    }

    /// Link two notes together
    pub fn link_notes(&self, note_id: &str, linked_note_id: &str) -> Result<()> {
        let mut note = self.get_note(note_id)?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;
        
        if !note.links.contains(&linked_note_id.to_string()) {
            note.links.push(linked_note_id.to_string());
            note.updated_at = chrono::Utc::now().to_rfc3339();
            
            // Save updated note
            let note_file = self.notes_dir.join(format!("{}.json", note.id));
            let note_json = serde_json::to_string_pretty(&note)?;
            std::fs::write(&note_file, note_json)?;
        }
        
        Ok(())
    }

    /// Unlink two notes
    pub fn unlink_notes(&self, note_id: &str, linked_note_id: &str) -> Result<()> {
        let mut note = self.get_note(note_id)?
            .ok_or_else(|| anyhow::anyhow!("Note not found: {}", note_id))?;
        
        note.links.retain(|id| id != linked_note_id);
        note.updated_at = chrono::Utc::now().to_rfc3339();
        
        // Save updated note
        let note_file = self.notes_dir.join(format!("{}.json", note.id));
        let note_json = serde_json::to_string_pretty(&note)?;
        std::fs::write(&note_file, note_json)?;
        
        Ok(())
    }

    /// Export note to markdown format
    pub fn export_note_to_markdown(&self, note: &Note) -> String {
        let mut md = String::new();
        
        // Title
        md.push_str(&format!("# {}\n\n", note.title));
        
        // Metadata
        md.push_str("---\n");
        md.push_str(&format!("**ID:** {}\n", note.id));
        md.push_str(&format!("**Created:** {}\n", note.created_at));
        md.push_str(&format!("**Updated:** {}\n", note.updated_at));
        
        if !note.tags.is_empty() {
            md.push_str(&format!("**Tags:** {}\n", note.tags.join(", ")));
        }
        
        if !note.links.is_empty() {
            md.push_str("**Links:** ");
            let link_titles: Vec<String> = note.links
                .iter()
                .filter_map(|link_id| {
                    self.get_note(link_id).ok().flatten()
                        .map(|n| format!("[[{}]]", n.title))
                })
                .collect();
            md.push_str(&link_titles.join(", "));
            md.push('\n');
        }
        
        md.push_str("---\n\n");
        
        // Content
        md.push_str(&note.content);
        md.push('\n');
        
        md
    }

    /// Get commit history for a note
    pub fn get_note_history(&self, note_id: &str) -> Result<Vec<crate::storage::CommitInfo>> {
        // Get the note to extract its title for matching
        let note = self.get_note(note_id)?;
        let note_title = note.as_ref().map(|n| n.title.as_str()).unwrap_or("");
        
        // Get the full path to the note file
        let note_file = self.notes_dir.join(format!("{}.json", note_id));
        let note_file_str = note_file.to_string_lossy().to_string();
        self.jujutsu.get_file_history_with_title(&note_file_str, note_title)
    }

    /// Get statistics about the knowledge base
    pub fn get_statistics(&self) -> Result<NoteStatistics> {
        let all_notes = self.list_notes()?;
        let total_notes = all_notes.len();
        let total_links: usize = all_notes.iter().map(|n| n.links.len()).sum();
        let total_tags: usize = all_notes.iter().map(|n| n.tags.len()).sum();
        let unique_tags: std::collections::HashSet<String> = all_notes
            .iter()
            .flat_map(|n| n.tags.iter().map(|t| t.to_lowercase()))
            .collect();
        
        Ok(NoteStatistics {
            total_notes,
            total_links,
            total_tags,
            unique_tags_count: unique_tags.len(),
        })
    }
}

#[derive(Debug)]
pub struct NoteStatistics {
    pub total_notes: usize,
    pub total_links: usize,
    pub total_tags: usize,
    pub unique_tags_count: usize,
}


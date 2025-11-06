use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub links: Vec<String>, // IDs of linked notes
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Note {
    /// Create a new note with a unique ID generated from title and timestamp
    pub fn new(title: String, content: String) -> Self {
        // Generate unique ID using MD5 hash of title and nanosecond timestamp
        let id = format!(
            "{:x}",
            md5::compute(format!("{}{}", title, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)))
        );
        let now = chrono::Utc::now().to_rfc3339();

        Note {
            id,
            title,
            content,
            links: Vec::new(),
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

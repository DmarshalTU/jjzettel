use crate::storage::note::Note;
use crate::service::note_service::NoteService;
use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub enum AppMode {
    List,
    View,
    Edit,
    Create,
    Search,
    DeleteConfirm,
    LinkSelect,
    TagAdd,
}

pub struct App {
    pub service: NoteService,
    pub notes: Vec<Note>,
    pub filtered_notes: Vec<Note>,
    pub is_searching: bool,
    pub search_query: String,
    pub selected_index: usize,
    pub link_selected_index: usize,
    pub mode: AppMode,
    pub current_note: Option<Note>,
    pub input_buffer: String,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let repo_path = std::env::var("JJZETTEL_REPO").unwrap_or_else(|_| "./jjzettel_repo".to_string());
        let service = NoteService::new(&repo_path);
        service.initialize()?;
        
        let notes = service.list_notes()?;
        
        let filtered_notes = notes.clone();
        
        Ok(App {
            service,
            notes,
            filtered_notes,
            is_searching: false,
            search_query: String::new(),
            selected_index: 0,
            link_selected_index: 0,
            mode: AppMode::List,
            current_note: None,
            input_buffer: String::new(),
            should_quit: false,
        })
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<()> {
        match self.mode {
            AppMode::List => self.handle_list_key(key)?,
            AppMode::View => self.handle_view_key(key)?,
            AppMode::Edit => self.handle_edit_key(key, modifiers)?,
            AppMode::Create => self.handle_create_key(key, modifiers)?,
            AppMode::Search => self.handle_search_key(key)?,
            AppMode::DeleteConfirm => self.handle_delete_confirm_key(key)?,
            AppMode::LinkSelect => self.handle_link_select_key(key)?,
            AppMode::TagAdd => self.handle_tag_add_key(key)?,
        }
        Ok(())
    }

    fn handle_list_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                if self.is_searching {
                    // Clear search
                    self.is_searching = false;
                    self.search_query.clear();
                    self.filtered_notes = self.notes.clone();
                    self.selected_index = 0;
                } else {
                    self.should_quit = true;
                }
            }
            crossterm::event::KeyCode::Char('/') => {
                // Start search
                self.mode = AppMode::Search;
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Char('#') => {
                // Start tag search
                self.mode = AppMode::Search;
                self.input_buffer = String::new();
                self.input_buffer.push('#');
            }
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                let max_index = if self.is_searching {
                    self.filtered_notes.len().saturating_sub(1)
                } else {
                    self.notes.len().saturating_sub(1)
                };
                if self.selected_index < max_index {
                    self.selected_index += 1;
                }
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            crossterm::event::KeyCode::Char('n') => {
                self.mode = AppMode::Create;
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Char('d') => {
                // Delete note
                let notes_to_use = if self.is_searching { &self.filtered_notes } else { &self.notes };
                if let Some(note) = notes_to_use.get(self.selected_index) {
                    self.current_note = Some(note.clone());
                    self.mode = AppMode::DeleteConfirm;
                }
            }
            crossterm::event::KeyCode::Enter => {
                let notes_to_use = if self.is_searching { &self.filtered_notes } else { &self.notes };
                if let Some(note) = notes_to_use.get(self.selected_index) {
                    self.current_note = Some(note.clone());
                    self.mode = AppMode::View;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_view_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::List;
                self.current_note = None;
                self.link_selected_index = 0;
            }
            crossterm::event::KeyCode::Char('e') => {
                self.mode = AppMode::Edit;
                if let Some(ref note) = self.current_note {
                    self.input_buffer = note.content.clone();
                }
            }
            crossterm::event::KeyCode::Char('l') => {
                // Link to another note
                self.mode = AppMode::LinkSelect;
                self.selected_index = 0;
            }
            crossterm::event::KeyCode::Char('t') => {
                // Add tag
                self.mode = AppMode::TagAdd;
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                // Navigate linked notes if available
                if let Some(ref note) = self.current_note {
                    if !note.links.is_empty() {
                        let max_index = note.links.len().saturating_sub(1);
                        if self.link_selected_index < max_index {
                            self.link_selected_index += 1;
                        }
                    }
                }
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                // Navigate linked notes if available
                if let Some(ref note) = self.current_note {
                    if !note.links.is_empty() && self.link_selected_index > 0 {
                        self.link_selected_index -= 1;
                    }
                }
            }
            crossterm::event::KeyCode::Enter => {
                // Navigate to selected linked note
                if let Some(ref note) = self.current_note {
                    if let Some(link_id) = note.links.get(self.link_selected_index) {
                        if let Ok(Some(linked_note)) = self.service.get_note(link_id) {
                            self.current_note = Some(linked_note);
                            self.link_selected_index = 0; // Reset selection
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::View;
            }
            crossterm::event::KeyCode::Char('s') if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+S to save
                if let Some(ref mut note) = self.current_note {
                    *note = self.service.update_note(note.clone(), self.input_buffer.clone())?;
                    self.mode = AppMode::View;
                    // Refresh notes list
                    self.notes = self.service.list_notes()?;
                    if self.is_searching {
                        self.filtered_notes = self.service.search_notes(&self.search_query)?;
                    } else {
                        self.filtered_notes = self.notes.clone();
                    }
                }
            }
            crossterm::event::KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            crossterm::event::KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            crossterm::event::KeyCode::Enter => {
                self.input_buffer.push('\n');
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_search_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::List;
                self.input_buffer.clear();
                self.is_searching = false;
                self.search_query.clear();
                self.filtered_notes = self.notes.clone();
                self.selected_index = 0;
            }
            crossterm::event::KeyCode::Enter => {
                // Apply search
                if self.input_buffer.trim().is_empty() {
                    self.is_searching = false;
                    self.search_query.clear();
                    self.filtered_notes = self.notes.clone();
                } else {
                    self.search_query = self.input_buffer.clone();
                    self.filtered_notes = self.service.search_notes(&self.input_buffer)?;
                    self.is_searching = true;
                }
                self.selected_index = 0;
                self.input_buffer.clear();
                self.mode = AppMode::List;
            }
            crossterm::event::KeyCode::Char(c) => {
                self.input_buffer.push(c);
                // Live search as you type
                if !self.input_buffer.trim().is_empty() {
                    self.filtered_notes = self.service.search_notes(&self.input_buffer)?;
                    self.is_searching = true;
                } else {
                    self.filtered_notes = self.notes.clone();
                    self.is_searching = false;
                }
                self.selected_index = 0;
            }
            crossterm::event::KeyCode::Backspace => {
                self.input_buffer.pop();
                // Live search as you type
                if !self.input_buffer.trim().is_empty() {
                    self.filtered_notes = self.service.search_notes(&self.input_buffer)?;
                    self.is_searching = true;
                } else {
                    self.filtered_notes = self.notes.clone();
                    self.is_searching = false;
                }
                self.selected_index = 0;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_delete_confirm_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Enter => {
                // Confirm deletion
                if let Some(ref note) = self.current_note {
                    self.service.delete_note(&note.id)?;
                    // Refresh notes
                    self.notes = self.service.list_notes()?;
                    if self.is_searching {
                        self.filtered_notes = self.service.search_notes(&self.search_query)?;
                    } else {
                        self.filtered_notes = self.notes.clone();
                    }
                    // Adjust selected index
                    if self.selected_index >= self.filtered_notes.len() && !self.filtered_notes.is_empty() {
                        self.selected_index = self.filtered_notes.len() - 1;
                    }
                }
                self.mode = AppMode::List;
                self.current_note = None;
            }
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('n') => {
                // Cancel deletion
                self.mode = AppMode::List;
                self.current_note = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_link_select_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::View;
            }
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                let max_index = self.notes.len().saturating_sub(1);
                if self.selected_index < max_index {
                    self.selected_index += 1;
                }
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            crossterm::event::KeyCode::Enter => {
                // Link current note to selected note
                if let Some(ref current_note) = self.current_note {
                    if let Some(target_note) = self.notes.get(self.selected_index) {
                        if current_note.id != target_note.id {
                            self.service.link_notes(&current_note.id, &target_note.id)?;
                            // Refresh current note
                            if let Some(updated_note) = self.service.get_note(&current_note.id)? {
                                self.current_note = Some(updated_note);
                            }
                            // Refresh notes list
                            self.notes = self.service.list_notes()?;
                            if self.is_searching {
                                self.filtered_notes = self.service.search_notes(&self.search_query)?;
                            } else {
                                self.filtered_notes = self.notes.clone();
                            }
                        }
                    }
                }
                self.mode = AppMode::View;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tag_add_key(&mut self, key: crossterm::event::KeyCode) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::View;
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Enter => {
                // Add tag
                if let Some(ref mut note) = self.current_note {
                    let tag = self.input_buffer.trim().to_string();
                    if !tag.is_empty() {
                        let updated_note = self.service.add_tag(&note.id, tag)?;
                        self.current_note = Some(updated_note);
                        // Refresh notes list
                        self.notes = self.service.list_notes()?;
                        if self.is_searching {
                            self.filtered_notes = self.service.search_notes(&self.search_query)?;
                        } else {
                            self.filtered_notes = self.notes.clone();
                        }
                    }
                }
                self.input_buffer = String::new();
                self.mode = AppMode::View;
            }
            crossterm::event::KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            crossterm::event::KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_create_key(&mut self, key: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> Result<()> {
        match key {
            crossterm::event::KeyCode::Esc => {
                self.mode = AppMode::List;
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Char('s') if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+S to save/create note
                if self.input_buffer.trim().is_empty() {
                    return Ok(());
                }
                
                // Create note with title from first line, content from entire buffer
                let lines: Vec<&str> = self.input_buffer.lines().collect();
                let title = lines.first().map(|s| s.to_string()).unwrap_or_else(|| "Untitled".to_string());
                let content = self.input_buffer.clone();
                
                let note = self.service.create_note(title, content)?;
                self.notes = self.service.list_notes()?;
                if self.is_searching {
                    self.filtered_notes = self.service.search_notes(&self.search_query)?;
                } else {
                    self.filtered_notes = self.notes.clone();
                }
                self.mode = AppMode::View;
                self.current_note = Some(note);
                self.input_buffer = String::new();
            }
            crossterm::event::KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            crossterm::event::KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            crossterm::event::KeyCode::Enter => {
                self.input_buffer.push('\n');
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame) {
        match self.mode {
            AppMode::List => self.render_list(frame),
            AppMode::View => self.render_view(frame),
            AppMode::Edit => self.render_edit(frame),
            AppMode::Create => self.render_create(frame),
            AppMode::Search => self.render_search(frame),
            AppMode::DeleteConfirm => self.render_delete_confirm(frame),
            AppMode::LinkSelect => self.render_link_select(frame),
            AppMode::TagAdd => self.render_tag_add(frame),
        }
    }

    fn render_list(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title_text = if self.is_searching {
            format!("jjzettel - Corporate Second Brain (Search: {})", self.search_query)
        } else {
            "jjzettel - Corporate Second Brain".to_string()
        };
        let title = Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Notes list
        let notes_to_display = if self.is_searching { &self.filtered_notes } else { &self.notes };
        let items: Vec<ListItem> = notes_to_display
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{} - {}", note.title, note.created_at)).style(style)
            })
            .collect();

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        
        let list_title = if self.is_searching {
            format!("Notes ({} found)", notes_to_display.len())
        } else {
            "Notes".to_string()
        };
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(list_title))
            .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_stateful_widget(list, chunks[1], &mut state);

        // Help bar
        let help = Paragraph::new("j/k: navigate | n: new | /: search | #: tag search | d: delete | Enter: view | Esc: quit")
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_view(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Note content with links and tags
        if let Some(ref note) = self.current_note {
            let mut content_lines: Vec<String> = note.content.lines().map(|s| s.to_string()).collect();
            
            // Add tags section if there are any
            if !note.tags.is_empty() {
                content_lines.push(String::new());
                content_lines.push("--- Tags ---".to_string());
                content_lines.push(note.tags.join(", "));
            }
            
            // Add links section if there are any
            if !note.links.is_empty() {
                content_lines.push(String::new());
                content_lines.push("--- Linked Notes ---".to_string());
                for (i, link_id) in note.links.iter().enumerate() {
                    if let Ok(Some(linked_note)) = self.service.get_note(link_id) {
                        let prefix = if i == self.link_selected_index {
                            "→ "
                        } else {
                            "  "
                        };
                        content_lines.push(format!("{}{}", prefix, linked_note.title));
                    }
                }
            }
            
            let content_text = content_lines.join("\n");
            let content = Paragraph::new(content_text)
                .block(Block::default().borders(Borders::ALL).title(note.title.as_str()))
                .wrap(Wrap { trim: true });
            frame.render_widget(content, chunks[1]);
        }

        // Help bar
        let help_text = if let Some(ref note) = self.current_note {
            if !note.links.is_empty() {
                "e: edit | l: link | t: tag | j/k: navigate links | Enter: open link | Esc: back"
            } else {
                "e: edit | l: link | t: tag | Esc: back"
            }
        } else {
            "e: edit | l: link | t: tag | Esc: back"
        };
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_edit(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Edit content
        let content = Paragraph::new(self.input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title("Editing"))
            .wrap(Wrap { trim: true });
        frame.render_widget(content, chunks[1]);

        // Help bar
        let help = Paragraph::new("Ctrl+S: save | Esc: cancel")
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_create(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Create content
        let content = Paragraph::new(self.input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title("New Note (first line is title)"))
            .wrap(Wrap { trim: true });
        frame.render_widget(content, chunks[1]);

        // Help bar
        let help = Paragraph::new("Ctrl+S: create | Esc: cancel")
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_search(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Search input
        let search_prompt = format!("Search: {}", self.input_buffer);
        let search = Paragraph::new(search_prompt.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(search, chunks[1]);

        // Results preview
        let results_text = if self.filtered_notes.is_empty() {
            "No results".to_string()
        } else {
            format!("{} results found", self.filtered_notes.len())
        };
        let results = Paragraph::new(results_text)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .wrap(Wrap { trim: true });
        frame.render_widget(results, chunks[2]);
    }

    fn render_delete_confirm(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Confirmation message
        let message = if let Some(ref note) = self.current_note {
            format!("Delete note: {}?\n\nPress Enter/y to confirm, Esc/n to cancel", note.title)
        } else {
            "Delete note?".to_string()
        };
        let confirm = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title("Confirm Delete"))
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Red));
        frame.render_widget(confirm, chunks[1]);

        // Help bar
        let help = Paragraph::new("Enter/y: confirm | Esc/n: cancel")
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_link_select(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Notes list for linking
        let items: Vec<ListItem> = self
            .notes
            .iter()
            .enumerate()
            .map(|(i, note)| {
                let style = if i == self.selected_index {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                // Show if already linked
                let already_linked = if let Some(ref current) = self.current_note {
                    current.links.contains(&note.id)
                } else {
                    false
                };
                let prefix = if already_linked { "✓ " } else { "  " };
                ListItem::new(format!("{}{} - {}", prefix, note.title, note.created_at)).style(style)
            })
            .collect();

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Select Note to Link"))
            .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_stateful_widget(list, chunks[1], &mut state);

        // Help bar
        let help = Paragraph::new("j/k: navigate | Enter: link | Esc: cancel")
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, chunks[2]);
    }

    fn render_tag_add(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(frame.area());

        // Title bar
        let title = Paragraph::new("jjzettel - Corporate Second Brain")
            .block(Block::default().borders(Borders::ALL).title("jjzettel"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Tag input
        let tag_prompt = format!("Tag: {}", self.input_buffer);
        let tag_input = Paragraph::new(tag_prompt.as_str())
            .block(Block::default().borders(Borders::ALL).title("Add Tag"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(tag_input, chunks[1]);

        // Current tags
        let tags_text = if let Some(ref note) = self.current_note {
            if note.tags.is_empty() {
                "No tags yet".to_string()
            } else {
                format!("Current tags: {}", note.tags.join(", "))
            }
        } else {
            String::new()
        };
        let tags = Paragraph::new(tags_text)
            .block(Block::default().borders(Borders::ALL).title("Tags"))
            .wrap(Wrap { trim: true });
        frame.render_widget(tags, chunks[2]);
    }
}


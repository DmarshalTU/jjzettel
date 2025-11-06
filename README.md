# jjzettel

A corporate second brain built with Rust, featuring a Zettelkasten-style note-taking system powered by [Jujutsu](https://github.com/martinvonz/jj) version control.

## 🧠 What is jjzettel?

jjzettel is a terminal-based knowledge management system designed for teams. It combines the power of Zettelkasten note-taking methodology with Jujutsu's modern version control system. Every note is a commit, and every link between notes creates a knowledge graph that grows with your organization.

**Simple Architecture**: No server or middleware needed. Each client directly accesses a shared Jujutsu repository, making collaboration natural and distributed.

## ✨ Features

- **📝 Note Management**: Create, edit, view, and delete notes
- **🔍 Search**: Full-text search across all notes
- **🔗 Note Linking**: Build knowledge graphs by linking related notes (Zettelkasten-style)
- **🏷️ Tags**: Organize notes with tags
- **📊 Version Control**: Every note change is tracked via Jujutsu commits
- **🎨 TUI Interface**: Beautiful terminal user interface built with Ratatui
- **⚡ Fast**: Built with Rust for performance

## 🚀 Installation

### Prerequisites

- Rust (latest stable version)
- [Jujutsu](https://github.com/martinvonz/jj) - Install via your package manager or from source

```bash
# Build from source
git clone https://github.com/martinvonz/jj.git
cd jj
cargo install --path cli
```

### Build jjzettel

```bash
git clone https://github.com/DmarshalTU/jjzettel.git
cd jjzettel
cargo build --release
```

The binary will be in `target/release/jjzettel`.

## 📖 Usage

### Running the Application

```bash
cargo run
# or
./target/release/jjzettel
```

### Repository Location

#### Local Repository (Default)

By default, jjzettel creates a local repository in `~/.jjzettel` (home directory) on first use. You can customize this by setting the `JJZETTEL_REPO` environment variable:

```bash
# Use default local repo (~/.jjzettel)
cargo run

# Use custom local repo path
export JJZETTEL_REPO=/path/to/your/repo
cargo run

# Or specify inline
JJZETTEL_REPO=/path/to/repo cargo run
```

**First-time use:** The repository is automatically initialized if it doesn't exist. Just run the app and it will create the repo for you.

#### Remote Repository (Git Sync)

To use a remote Git repository for collaboration:

1. **Clone an existing Git repository:**
   ```bash
   git clone https://github.com/your-org/knowledge-base.git
   cd knowledge-base
   jj init --git .
   export JJZETTEL_REPO=$(pwd)
   cargo run
   ```

2. **Create a new repo and push to Git:**
   ```bash
   # Create local repo
   mkdir my-knowledge-base
   cd my-knowledge-base
   jj init --git .
   
   # Add Git remote
   git remote add origin https://github.com/your-org/knowledge-base.git
   
   # Use it
   export JJZETTEL_REPO=$(pwd)
   cargo run
   
   # Push to remote
   jj git push
   ```

3. **Sync with remote:**
   ```bash
   # Pull latest changes
   jj git pull
   
   # Push your changes
   jj git push
   ```

#### Shared Network Repository

For teams using shared network storage (NFS, SMB, etc.):

```bash
# Point to shared network path
export JJZETTEL_REPO=/mnt/shared/company-knowledge-base
cargo run
```

All team members point to the same path, and Jujutsu handles concurrent access automatically.

## ⌨️ Keybindings

### List Mode
- `j` / `↓` - Navigate down
- `k` / `↑` - Navigate up
- `n` - Create new note
- `/` - Search notes
- `d` - Delete selected note
- `Enter` - View note
- `Esc` - Quit (or clear search)

### View Mode
- `e` - Edit note
- `l` - Link to another note
- `t` - Add tag
- `Esc` - Back to list

### Edit/Create Mode
- Type to edit content
- `Ctrl+S` - Save
- `Esc` - Cancel

### Search Mode
- Type to search (live search)
- `Enter` - Apply search
- `Esc` - Cancel

### Link Select Mode
- `j/k` - Navigate notes
- `Enter` - Create link
- `Esc` - Cancel

### Tag Add Mode
- Type tag name
- `Enter` - Add tag
- `Esc` - Cancel

### Delete Confirm
- `Enter` / `y` - Confirm deletion
- `Esc` / `n` - Cancel

## 🏗️ Architecture

```
jjzettel/
├── src/
│   ├── main.rs              # Entry point, TUI setup
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── note.rs          # Note data structure
│   │   └── jujutsu.rs       # Jujutsu wrapper
│   ├── service/
│   │   ├── mod.rs
│   │   └── note_service.rs  # Business logic
│   └── tui/
│       ├── mod.rs
│       └── app.rs           # TUI application state
```

### Components

1. **TUI Layer** (`tui/`): Terminal user interface and state management
2. **Service Layer** (`service/`): Business logic for notes, tags, links
3. **Storage Layer** (`storage/`): Handles note persistence and Jujutsu integration

### Multi-User & Collaboration

Since jjzettel uses Jujutsu as its storage backend, collaboration is built-in:

**Workflow Summary:**

1. **Local-only (default):**
   - First run: Creates `~/.jjzettel` automatically
   - Subsequent runs: Uses existing repo
   - Perfect for personal use

2. **Remote Git (collaboration):**
   - Clone existing repo or create new one
   - Initialize with `jj init --git .`
   - Set `JJZETTEL_REPO` to the repo path
   - Sync with `jj git push/pull` as needed

3. **Shared Network (team):**
   - All team members set `JJZETTEL_REPO` to same network path
   - Jujutsu handles concurrent access
   - No manual sync needed

**Notes:**
- If `JJZETTEL_REPO` is not set, defaults to `~/.jjzettel` (home directory)
- Repository is auto-initialized if it doesn't exist
- For Git remotes, use `jj git push/pull` commands outside the app
- The app doesn't manage Git remotes - use Jujutsu CLI for that

## 📝 Note Format

Notes are stored as JSON files with the following structure:

```json
{
  "id": "unique-note-id",
  "title": "Note Title",
  "content": "Note content...",
  "links": ["linked-note-id-1", "linked-note-id-2"],
  "tags": ["tag1", "tag2"],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## 🔗 Jujutsu Integration

Each note operation creates a Jujutsu commit:
- Creating a note → "Note: {title}" commit
- Updating a note → "Update: {title}" commit
- Deleting a note → "Delete note: {id}" commit

This gives you:
- Full version history of every note
- Branching and merging capabilities
- Easy collaboration via remote repositories
- Time-travel through your knowledge base

## 🎯 Zettelkasten Methodology

jjzettel follows Zettelkasten principles:

1. **Atomic Notes**: Each note should be about one concept
2. **Linking**: Notes are connected to form a knowledge graph
3. **Tags**: Additional organization through tags
4. **Permanent Notes**: Notes are never deleted, only archived (via Jujutsu)
5. **Backlinks**: Automatically see which notes link to the current note

## 🔄 Collaboration Model

Unlike traditional note-taking apps that require a server, jjzettel uses a simple peer-to-peer model:

```
Client 1 ──┐
           ├──> Shared Jujutsu Repo <── Client 2
Client 3 ──┘                    └── Client 4
```

**Benefits:**
- ✅ No server to maintain
- ✅ Works offline (Jujutsu handles sync when online)
- ✅ Automatic conflict resolution
- ✅ Full version history
- ✅ Can use Git remotes for backup/sync
- ✅ Works with shared network storage (NFS, SMB, etc.)

## 🛠️ Development

### Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `tokio` - Async runtime
- `serde` - Serialization
- `anyhow` - Error handling
- `chrono` - Date/time handling
- `md5` - Note ID generation

### Building

```bash
cargo build
cargo build --release
```

### Running Tests

```bash
cargo test
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 🙏 Acknowledgments

- [Jujutsu](https://github.com/martinvonz/jj) - The version control system that makes this possible
- [Ratatui](https://github.com/ratatui-org/ratatui) - Beautiful terminal UI framework
- Zettelkasten methodology inspiration

## 📧 Contact

Created by DmarshalTU

---

**Happy note-taking! 🚀**


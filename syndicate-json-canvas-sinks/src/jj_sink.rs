use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::Local;
use tracing::{debug, info};
use syndicate_json_canvas_lib::SyndicationFormat;
use crate::{SinkError, SyndicationSink};

/// Configuration for JJ repository syndication sink
pub struct JjRepositorySink {
    /// Path to the JJ repository
    repo_path: PathBuf,
    /// Bookmark name to update (e.g., "main")
    bookmark_name: String,
    /// Remote name to push to (e.g., "origin")
    remote_name: String,
    /// Folder path within the repository to put files in
    folder_path: PathBuf,
}

impl JjRepositorySink {
    /// Create a new JJ repository sink
    ///
    /// # Arguments
    /// * `repo_path` - Path to the JJ repository root
    /// * `bookmark_name` - Bookmark to update (default: "main")
    /// * `remote_name` - Remote to push to (default: "origin")
    /// * `folder_path` - Folder within repo for microblog files
    pub fn new(
        repo_path: impl AsRef<Path>,
        bookmark_name: impl Into<String>,
        remote_name: impl Into<String>,
        folder_path: impl AsRef<Path>,
    ) -> Result<Self, SinkError> {
        let repo_path = repo_path.as_ref().to_path_buf();

        // Validate that the path exists and is a directory
        if !repo_path.exists() {
            return Err(SinkError::Config(format!(
                "Repository path does not exist: {}",
                repo_path.display()
            )));
        }

        if !repo_path.is_dir() {
            return Err(SinkError::Config(format!(
                "Repository path is not a directory: {}",
                repo_path.display()
            )));
        }

        Ok(Self {
            repo_path,
            bookmark_name: bookmark_name.into(),
            remote_name: remote_name.into(),
            folder_path: folder_path.as_ref().to_path_buf(),
        })
    }

    /// Generate a slug from the content text (first 8 words)
    fn generate_slug(text: &str) -> String {
        text.split_whitespace()
            .take(8)
            .map(|word| {
                // Remove punctuation and convert to lowercase
                word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Generate the filename for a syndication item
    fn generate_filename(item: &SyndicationFormat) -> String {
        let slug = Self::generate_slug(&item.text);
        let node_id = item.id.as_str();
        format!("{}-{}.md", slug, node_id)
    }

    /// Generate file contents with frontmatter
    fn generate_file_contents(item: &SyndicationFormat) -> String {
        let slug = Self::generate_slug(&item.text);
        let date = Local::now().format("%Y-%m-%d").to_string();

        format!(
            "---\ntitle: \"{}\"\ndate: {}\n---\n\n{}",
            slug, date, item.text
        )
    }

    /// Generate commit message
    fn generate_commit_message(item: &SyndicationFormat) -> String {
        let slug = Self::generate_slug(&item.text);
        let preview = if item.text.len() > 50 {
            format!("{}...", &item.text[..50])
        } else {
            item.text.clone()
        };

        format!("Adding microblog `{}`\n\n{}", slug, preview)
    }

    /// Run a JJ command in the repository
    fn run_jj_command(&self, args: &[&str], dry_run: bool) -> Result<String, SinkError> {
        let args_str = args.join(" ");

        if dry_run {
            debug!(command = %format!("jj {}", args_str), "[DRY RUN] Would execute command");
            return Ok(String::new());
        }

        debug!(command = %format!("jj {}", args_str), "Executing command");

        let output = Command::new("jj")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| SinkError::CommandFailed(format!("Failed to execute jj: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SinkError::CommandFailed(format!(
                "jj {} failed: {}",
                args_str, stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Write the file to the repository
    fn write_file(&self, filename: &str, contents: &str, dry_run: bool) -> Result<(), SinkError> {
        let file_path = self.repo_path.join(&self.folder_path).join(filename);

        if dry_run {
            debug!(
                file = %file_path.display(),
                contents = %contents,
                "[DRY RUN] Would write file"
            );
            return Ok(());
        }

        // Ensure the folder exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&file_path, contents)?;
        debug!(file = %file_path.display(), "Wrote file");

        Ok(())
    }
}

impl SyndicationSink for JjRepositorySink {
    fn publish(&mut self, item: &SyndicationFormat, dry_run: bool) -> Result<(), SinkError> {
        info!("Publishing to JJ repository");

        // Step 1: jj git fetch
        self.run_jj_command(&["git", "fetch"], dry_run)?;

        // Step 2: Generate content
        let filename = Self::generate_filename(item);
        let contents = Self::generate_file_contents(item);
        let commit_message = Self::generate_commit_message(item);

        debug!(
            filename = %filename,
            slug = %Self::generate_slug(&item.text),
            "Generated content"
        );

        // Step 3: jj new --insert-after <bookmark> -m <message>
        self.run_jj_command(
            &[
                "new",
                "--insert-after",
                &self.bookmark_name,
                "-m",
                &commit_message,
            ],
            dry_run,
        )?;

        // Step 4: Write the file
        self.write_file(&filename, &contents, dry_run)?;

        // Step 5: jj bookmark move <bookmark>
        self.run_jj_command(&["bookmark", "move", &self.bookmark_name], dry_run)?;

        // Step 6: jj git push --remote <remote> --bookmark <bookmark>
        self.run_jj_command(
            &[
                "git",
                "push",
                "--remote",
                &self.remote_name,
                "--bookmark",
                &self.bookmark_name,
            ],
            dry_run,
        )?;

        info!("Successfully published to JJ repository");
        Ok(())
    }

    fn name(&self) -> &str {
        "JJ Repository"
    }
}

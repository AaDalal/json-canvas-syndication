use crate::{SinkError, SyndicationSink};
use chrono::Local;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use syndicate_json_canvas_lib::{SyndicationFormat, jsoncanvas::NodeId};
use tracing::{debug, info};

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
    fn generate_filename(slug: &str, node_id: &NodeId) -> String {
        format!("{}-{}.md", slug, node_id.as_str())
    }

    /// Escape double quotes and backslashes for YAML string values
    fn escape_yaml_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    /// Generate file contents with frontmatter including cross-references
    fn generate_file_contents(
        item: &SyndicationFormat,
        _slug: &str,
        slugs: &HashMap<NodeId, String>,
        all_items: &HashMap<NodeId, SyndicationFormat>,
    ) -> String {
        let date = Local::now().format("%Y-%m-%d").to_string();

        // Use first 8 words for title, or full text if shorter
        let title: String = item.text
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ");

        // Build context_for_this list (in-neighbors with /t/ prefix)
        // Each item is an object with link_text and href
        let context_for_this: Vec<(String, String)> = item.in_neighbor_ids
            .iter()
            .filter_map(|node_id| {
                let neighbor_slug = slugs.get(node_id)?;
                let neighbor_item = all_items.get(node_id)?;
                let link_text: String = neighbor_item.text
                    .split_whitespace()
                    .take(8)
                    .collect::<Vec<_>>()
                    .join(" ");
                let href = format!("/t/{}-{}.md", neighbor_slug, node_id.as_str());
                Some((link_text, href))
            })
            .collect();

        // Build further_thinking list (out-neighbors with /t/ prefix)
        // Each item is an object with link_text and href
        let further_thinking: Vec<(String, String)> = item.out_neighbor_ids
            .iter()
            .filter_map(|node_id| {
                let neighbor_slug = slugs.get(node_id)?;
                let neighbor_item = all_items.get(node_id)?;
                let link_text: String = neighbor_item.text
                    .split_whitespace()
                    .take(8)
                    .collect::<Vec<_>>()
                    .join(" ");
                let href = format!("/t/{}-{}.md", neighbor_slug, node_id.as_str());
                Some((link_text, href))
            })
            .collect();

        // Format frontmatter with escaped strings
        let mut frontmatter = format!(
            "---\ntitle: \"{}\"\ndate: {}\n",
            Self::escape_yaml_string(&title), date
        );

        if !context_for_this.is_empty() {
            frontmatter.push_str("context_for_this:\n");
            for (link_text, href) in context_for_this {
                frontmatter.push_str(&format!("  - link_text: \"{}\"\n", Self::escape_yaml_string(&link_text)));
                frontmatter.push_str(&format!("    href: \"{}\"\n", href));
            }
        }

        if !further_thinking.is_empty() {
            frontmatter.push_str("further_thinking:\n");
            for (link_text, href) in further_thinking {
                frontmatter.push_str(&format!("  - link_text: \"{}\"\n", Self::escape_yaml_string(&link_text)));
                frontmatter.push_str(&format!("    href: \"{}\"\n", href));
            }
        }

        frontmatter.push_str("---\n\n");

        format!("{}{}", frontmatter, item.text)
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
    fn publish(&mut self, items: &HashMap<NodeId, SyndicationFormat>, dry_run: bool) -> Result<(), SinkError> {
        info!(item_count = items.len(), "Publishing to JJ repository");

        if items.is_empty() {
            info!("No items to publish");
            return Ok(());
        }

        // Step 1: jj git fetch
        self.run_jj_command(&["git", "fetch"], dry_run)?;

        // Step 2: Pre-compute slugs for all items
        let slugs: HashMap<NodeId, String> = items
            .iter()
            .map(|(node_id, item)| (node_id.clone(), Self::generate_slug(&item.text)))
            .collect();

        // Step 3: Generate commit message
        let commit_message = if items.len() == 1 {
            let item = items.values().next().unwrap();
            let slug = slugs.get(&item.id).unwrap();
            let preview = if item.text.len() > 50 {
                format!("{}...", &item.text[..50])
            } else {
                item.text.clone()
            };
            format!("Adding microblog `{}`\n\n{}", slug, preview)
        } else {
            format!("Update microblogs ({} posts)", items.len())
        };

        // Step 4: jj new --insert-after <bookmark> -m <message>
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

        // Step 5: Write all files
        for (node_id, item) in items.iter() {
            let slug = slugs.get(node_id).unwrap();
            let filename = Self::generate_filename(slug, node_id);
            let contents = Self::generate_file_contents(item, slug, &slugs, items);

            debug!(
                filename = %filename,
                slug = %slug,
                "Generated content"
            );

            self.write_file(&filename, &contents, dry_run)?;
        }

        // Step 6: jj bookmark move <bookmark>
        self.run_jj_command(&["bookmark", "move", &self.bookmark_name], dry_run)?;

        // Step 7: jj git push --remote <remote> --bookmark <bookmark>
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
        "jj"
    }
}

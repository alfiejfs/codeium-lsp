use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Debug, Clone, PartialEq)]
pub struct ContentAnalysis {
    pub content_before: String,
    pub content_after: String,
    pub last_character: String,
    pub last_line: String,
    pub content_immediately_after: String,
    pub cursor_position: usize,
}

impl ContentAnalysis {
    /// Analyzes content at a specific line and column position
    ///
    /// # Arguments
    /// * `contents` - The full content string
    /// * `line` - Line number (0-based)
    /// * `column` - Column number (0-based)
    ///
    /// # Returns
    /// * `ContentAnalysis` - Analysis of the content at the specified position
    pub fn new(contents: &str, line: usize, column: usize) -> Self {
        let lines: Vec<&str> = contents.split('\n').collect();

        // Handle case where line is beyond the content
        if line >= lines.len() {
            return ContentAnalysis {
                content_before: contents.to_string(),
                content_after: String::new(),
                last_character: contents
                    .chars()
                    .last()
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
                last_line: lines.last().unwrap_or(&"").to_string(),
                content_immediately_after: String::new(),
                cursor_position: contents.chars().count(),
            };
        }

        let target_line = lines[line];
        let target_line_chars: Vec<char> = target_line.chars().collect();

        // Handle case where column is beyond the line length
        let effective_column = column.min(target_line_chars.len());

        // Build content_before: all complete lines before target + partial target line
        let mut content_before_parts = Vec::new();

        // Add all complete lines before the target line
        for i in 0..line {
            content_before_parts.push(lines[i].to_string());
        }

        // Add the partial target line (up to column position)
        let partial_target_line: String = target_line_chars.iter().take(effective_column).collect();
        content_before_parts.push(partial_target_line);

        let content_before = content_before_parts.join("\n");

        // Calculate cursor position (absolute character position in file)
        let cursor_position = content_before.chars().count();

        // Build content_after: remainder of target line + all lines after
        let mut content_after_parts = Vec::new();

        // Add remainder of target line (from column position onwards)
        let remainder_of_target_line: String =
            target_line_chars.iter().skip(effective_column).collect();
        content_after_parts.push(remainder_of_target_line);

        // Add all lines after the target line
        for i in (line + 1)..lines.len() {
            content_after_parts.push(lines[i].to_string());
        }

        let content_after = content_after_parts.join("\n");

        // Get the last character of content_before
        let last_character = content_before
            .chars()
            .last()
            .map(|c| c.to_string())
            .unwrap_or_default();

        // Get the last line (partial target line)
        let last_line = target_line_chars.iter().take(effective_column).collect();

        // Get content immediately after the column position on the same line
        let content_immediately_after: String =
            target_line_chars.iter().skip(effective_column).collect();

        ContentAnalysis {
            content_before,
            content_after,
            last_character,
            last_line,
            content_immediately_after,
            cursor_position,
        }
    }
}

pub async fn log(message: &str) {
    let path = "/Users/alfiejfs/codeium-log";
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
    {
        let _ = file.write_all(message.as_bytes()).await;
        let _ = file.write_all(b"\n").await;
    }
}

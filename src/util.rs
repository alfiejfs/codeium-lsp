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

        // Get lines up to and including the target line
        let mut lines_before: Vec<String> =
            lines.iter().take(line + 1).map(|s| s.to_string()).collect();

        // Truncate the last line at the column position
        if let Some(last_line) = lines_before.last_mut() {
            let chars: Vec<char> = last_line.chars().collect();
            *last_line = chars.iter().take(column).collect();
        }

        let last_line = lines_before.last().cloned().unwrap_or_default();
        let content_before = lines_before.join("\n");

        // Calculate cursor position (absolute character position in file)
        let cursor_position = content_before.chars().count();

        // Get content after the target line
        let content_after = lines
            .iter()
            .copied()
            .skip(line + 1)
            .collect::<Vec<_>>()
            .join("\n");

        // Gt the last character of content_before
        let last_character = content_before
            .chars()
            .last()
            .map(|c| c.to_string())
            .unwrap_or_default();

        // Get content immediately after the column position on the same line
        let content_immediately_after = lines
            .get(line)
            .map(|line_content| {
                let chars: Vec<char> = line_content.chars().collect();
                chars.iter().skip(column).collect::<String>()
            })
            .unwrap_or_default();

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

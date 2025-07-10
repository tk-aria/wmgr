use colored::{Color, Colorize};
use console::{Style, Term};
use indicatif::{ProgressBar, ProgressStyle, ProgressDrawTarget};
use std::io::{self, Write};
use std::time::Duration;

/// Display utilities for the CLI interface
pub struct DisplayHelper {
    pub use_color: bool,
    pub terminal: Term,
}

impl DisplayHelper {
    /// Create a new DisplayHelper
    pub fn new(use_color: bool) -> Self {
        Self {
            use_color,
            terminal: Term::stdout(),
        }
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        if self.use_color {
            println!("{} {}", "✓".green().bold(), message);
        } else {
            println!("[SUCCESS] {}", message);
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        if self.use_color {
            eprintln!("{} {}", "✗".red().bold(), message);
        } else {
            eprintln!("[ERROR] {}", message);
        }
    }

    /// Print a warning message
    pub fn warning(&self, message: &str) {
        if self.use_color {
            println!("{} {}", "⚠".yellow().bold(), message);
        } else {
            println!("[WARNING] {}", message);
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        if self.use_color {
            println!("{} {}", "::".blue().bold(), message);
        } else {
            println!("[INFO] {}", message);
        }
    }

    /// Print a debug message (only if verbose is enabled)
    pub fn debug(&self, message: &str, verbose: bool) {
        if verbose {
            if self.use_color {
                println!("{} {}", "->".dimmed(), message.dimmed());
            } else {
                println!("[DEBUG] {}", message);
            }
        }
    }

    /// Print a section header
    pub fn section_header(&self, title: &str) {
        if self.use_color {
            println!("\n{}", title.bold().underline());
        } else {
            println!("\n=== {} ===", title);
        }
    }

    /// Print a subsection header
    pub fn subsection_header(&self, title: &str) {
        if self.use_color {
            println!("  {}", title.bold());
        } else {
            println!("  -- {} --", title);
        }
    }

    /// Format a file path with appropriate styling
    pub fn format_path(&self, path: &str) -> String {
        if self.use_color {
            path.cyan().to_string()
        } else {
            format!("'{}'", path)
        }
    }

    /// Format a URL with appropriate styling
    pub fn format_url(&self, url: &str) -> String {
        if self.use_color {
            url.blue().underline().to_string()
        } else {
            url.to_string()
        }
    }

    /// Format a command with appropriate styling
    pub fn format_command(&self, command: &str) -> String {
        if self.use_color {
            command.magenta().bold().to_string()
        } else {
            format!("`{}`", command)
        }
    }

    /// Format a branch name with appropriate styling
    pub fn format_branch(&self, branch: &str) -> String {
        if self.use_color {
            branch.green().to_string()
        } else {
            format!("'{}'", branch)
        }
    }

    /// Format a repository name with appropriate styling
    pub fn format_repo(&self, repo: &str) -> String {
        if self.use_color {
            repo.cyan().bold().to_string()
        } else {
            repo.to_string()
        }
    }

    /// Create a progress bar for operations
    pub fn create_progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        let pb = if self.use_color {
            ProgressBar::new(len)
        } else {
            ProgressBar::with_draw_target(Some(len), ProgressDrawTarget::hidden())
        };

        if self.use_color {
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            pb.set_message(message.to_string());
        }

        pb
    }

    /// Create a spinner for indeterminate operations
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = if self.use_color {
            ProgressBar::new_spinner()
        } else {
            ProgressBar::with_draw_target(None, ProgressDrawTarget::hidden())
        };

        if self.use_color {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"])
                    .template("{spinner:.green} {msg}")
                    .unwrap()
            );
            pb.set_message(message.to_string());
            pb.enable_steady_tick(Duration::from_millis(120));
        }

        pb
    }

    /// Print a table-like structure
    pub fn print_table(&self, headers: &[&str], rows: &[Vec<String>]) {
        if rows.is_empty() {
            return;
        }

        // Calculate column widths
        let mut col_widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Print headers
        if self.use_color {
            for (i, header) in headers.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{:<width$}", header.bold(), width = col_widths[i]);
            }
        } else {
            for (i, header) in headers.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{:<width$}", header, width = col_widths[i]);
            }
        }
        println!();

        // Print separator
        if self.use_color {
            for (i, &width) in col_widths.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{}", "─".repeat(width));
            }
        } else {
            for (i, &width) in col_widths.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{}", "-".repeat(width));
            }
        }
        println!();

        // Print rows
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                let width = col_widths.get(i).unwrap_or(&0);
                print!("{:<width$}", cell, width = width);
            }
            println!();
        }
    }

    /// Print a status indicator
    pub fn print_status(&self, status: StatusType, message: &str) {
        let (icon, color) = match status {
            StatusType::Success => ("✓", Color::Green),
            StatusType::Error => ("✗", Color::Red),
            StatusType::Warning => ("⚠", Color::Yellow),
            StatusType::Info => ("::", Color::Blue),
            StatusType::Working => ("→", Color::Cyan),
        };

        if self.use_color {
            println!("{} {}", icon.color(color).bold(), message);
        } else {
            let label = match status {
                StatusType::Success => "[OK]",
                StatusType::Error => "[ERROR]",
                StatusType::Warning => "[WARN]",
                StatusType::Info => "[INFO]",
                StatusType::Working => "[WORK]",
            };
            println!("{} {}", label, message);
        }
    }

    /// Print a list with bullets
    pub fn print_list(&self, items: &[&str]) {
        for item in items {
            if self.use_color {
                println!("  {} {}", "•".blue(), item);
            } else {
                println!("  - {}", item);
            }
        }
    }

    /// Print an indented message
    pub fn print_indented(&self, message: &str, level: usize) {
        let indent = "  ".repeat(level);
        println!("{}{}", indent, message);
    }

    /// Clear the current line
    pub fn clear_line(&self) {
        if self.use_color {
            let _ = self.terminal.clear_line();
        }
    }

    /// Move cursor up
    pub fn move_cursor_up(&self, lines: usize) {
        if self.use_color {
            let _ = self.terminal.move_cursor_up(lines);
        }
    }

    /// Prompt for confirmation
    pub fn confirm(&self, message: &str) -> io::Result<bool> {
        if self.use_color {
            print!("{} {} [y/N]: ", "?".yellow().bold(), message);
        } else {
            print!("[CONFIRM] {} [y/N]: ", message);
        }
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }

    /// Print error details with context
    pub fn print_error_details(&self, title: &str, error: &anyhow::Error) {
        self.error(title);
        
        let mut current = error.source();
        let mut level = 1;
        
        while let Some(err) = current {
            self.print_indented(&format!("Caused by: {}", err), level);
            current = err.source();
            level += 1;
        }
    }

    /// Format a duration in human-readable format
    pub fn format_duration(&self, duration: Duration) -> String {
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        
        if secs > 60 {
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            format!("{}m {}s", mins, remaining_secs)
        } else if secs > 0 {
            format!("{}.{}s", secs, millis / 100)
        } else {
            format!("{}ms", millis)
        }
    }

    /// Print a summary box
    pub fn print_summary(&self, title: &str, items: &[(String, String)]) {
        if self.use_color {
            println!("\n┌─ {} ─┐", title.bold());
            for (key, value) in items {
                println!("│ {}: {}", key.bold(), value);
            }
            println!("└{:─<width$}┘", "", width = title.len() + 4);
        } else {
            println!("\n=== {} ===", title);
            for (key, value) in items {
                println!("{}: {}", key, value);
            }
            println!("{}", "=".repeat(title.len() + 8));
        }
    }
}

/// Status types for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    Success,
    Error,
    Warning,
    Info,
    Working,
}

/// Helper functions for common display patterns
pub mod helpers {
    use super::*;

    /// Create a display helper with color detection
    pub fn auto_display() -> DisplayHelper {
        let use_color = atty::is(atty::Stream::Stdout) && std::env::var("NO_COLOR").is_err();
        DisplayHelper::new(use_color)
    }

    /// Print a separator line
    pub fn print_separator(display: &DisplayHelper) {
        if display.use_color {
            println!("{}", "─".repeat(60).dimmed());
        } else {
            println!("{}", "-".repeat(60));
        }
    }

    /// Print a command execution result
    pub fn print_command_result(display: &DisplayHelper, command: &str, success: bool, output: Option<&str>) {
        if success {
            display.success(&format!("Command executed: {}", display.format_command(command)));
        } else {
            display.error(&format!("Command failed: {}", display.format_command(command)));
        }
        
        if let Some(output) = output {
            if !output.trim().is_empty() {
                println!("Output:");
                for line in output.lines() {
                    display.print_indented(line, 1);
                }
            }
        }
    }

    /// Print repository information
    pub fn print_repo_info(display: &DisplayHelper, name: &str, url: &str, branch: Option<&str>) {
        println!("{}: {}", display.format_repo(name), display.format_url(url));
        if let Some(branch) = branch {
            display.print_indented(&format!("Branch: {}", display.format_branch(branch)), 1);
        }
    }
}
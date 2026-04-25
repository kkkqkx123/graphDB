use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

use crate::completion::completer::GraphDBCompleter;
use crate::utils::error::{CliError, Result};

pub struct InputHandler {
    editor: Editor<GraphDBCompleter, DefaultHistory>,
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        let completer = GraphDBCompleter::new();
        let mut editor = Editor::new()
            .map_err(|e| CliError::Other(format!("Failed to create line editor: {}", e)))?;

        editor.set_helper(Some(completer));
        editor.set_auto_add_history(true);

        let history_path = get_history_path();
        if let Some(parent) = history_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = editor.load_history(&history_path);

        Ok(Self { editor })
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>> {
        match self.editor.readline(prompt) {
            Ok(line) => Ok(Some(line)),
            Err(ReadlineError::Interrupted) => Ok(None),
            Err(ReadlineError::Eof) => Ok(None),
            Err(e) => Err(CliError::Other(format!("Read error: {}", e))),
        }
    }

    pub fn save_history(&mut self) {
        let history_path = get_history_path();
        let _ = self.editor.save_history(&history_path);
    }
}

fn get_history_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".graphdb").join("cli_history")
}

pub fn is_statement_complete(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return true;
    }

    if trimmed.starts_with('\\') {
        return true;
    }

    if !trimmed.ends_with(';') {
        return false;
    }

    let mut paren_count = 0i32;
    let mut brace_count = 0i32;
    let mut bracket_count = 0i32;
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in trimmed.chars() {
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }

        match ch {
            '\'' | '"' => {
                in_string = true;
                string_char = ch;
            }
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            '[' => bracket_count += 1,
            ']' => bracket_count -= 1,
            _ => {}
        }
    }

    paren_count == 0 && brace_count == 0 && bracket_count == 0
}

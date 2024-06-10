use std::fmt::Display;

pub enum InputError {
    Eof,
    Interrupt,
    Other(String),
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Eof => write!(f, "End of file."),
            InputError::Interrupt => write!(f, "Keyboard interrupt."),
            InputError::Other(description) => write!(f, "{description}"),
        }
    }
}

pub struct Input {
    editor: rustyline::Editor<(), rustyline::history::MemHistory>,
}

impl Input {
    const PROMPT: &'static str = "> ";

    pub fn new() -> Self {
        let editor = rustyline::Editor::with_history(
            rustyline::Config::default(),
            rustyline::history::MemHistory::new(),
        )
        .expect("Failed to initialise terminal editor.");
        Self { editor }
    }

    fn readline(&mut self, prompt: &str) -> Result<String, InputError> {
        let line = match self.editor.readline(prompt) {
            Ok(line) => Ok(line),
            Err(rustyline::error::ReadlineError::WindowResized) => self.line(),
            Err(rustyline::error::ReadlineError::Eof) => Err(InputError::Eof),
            Err(rustyline::error::ReadlineError::Interrupted) => Err(InputError::Interrupt),
            Err(err) => Err(InputError::Other(err.to_string())),
        }?;

        (self.editor.history_mut() as &mut dyn rustyline::history::History)
            .add(&line)
            .ok();

        Ok(line)
    }

    pub fn prompt(&mut self, prompt: &str) -> Result<String, InputError> {
        self.readline(&format!("{prompt} {}", Self::PROMPT))
    }

    pub fn line(&mut self) -> Result<String, InputError> {
        self.readline(Self::PROMPT)
    }
}

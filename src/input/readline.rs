pub enum InputError {
    Eof,
    Interrupt,
    Other(String),
}

pub struct Input {
    editor: rustyline::DefaultEditor,
}

impl Input {
    const PROMPT: &str = "> ";

    pub fn new() -> Self {
        Self {
            editor: rustyline::DefaultEditor::new().unwrap(),
        }
    }

    pub fn line(&mut self) -> Result<String, InputError> {
        match self.editor.readline(Self::PROMPT) {
            Ok(line) => Ok(line),
            Err(rustyline::error::ReadlineError::WindowResized) => self.line(),
            Err(rustyline::error::ReadlineError::Eof) => Err(InputError::Eof),
            Err(rustyline::error::ReadlineError::Interrupted) => Err(InputError::Interrupt),
            Err(err) => Err(InputError::Other(err.to_string())),
        }
    }
}

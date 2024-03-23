use crate::{err, AppState, Res};

const COMMANDS: &[(&str, &'static dyn Fn(&[String], &mut AppState) -> Res<()>)] =
    &[("exit", &exit), ("save", &save), ("load", &load)];

fn single_opt_arg(args: &[String]) -> Res<Option<&str>> {
    match args {
        [] => Ok(None),
        [arg] => Ok(Some(arg)),
        _ => err("Expected at most one argument."),
    }
}

pub fn exit(args: &[String], state: &mut AppState) -> Res<()> {
    const NOSAVE_ARGS: &[&str] = &["!", "nosave"];
    const USAGE: &str = "Usage: exit [nosave]";

    let mut skip_save = false;
    if let Some(arg) = single_opt_arg(args)? {
        if NOSAVE_ARGS.contains(&arg) {
            skip_save = true
        } else {
            return err(USAGE);
        }
    }

    if !skip_save {
        save(&[], state)?;
    }

    std::process::exit(0);
}

fn save(args: &[String], state: &mut AppState) -> Res<()> {
    let title = if let Some(arg) = single_opt_arg(args)? {
        Some(arg.to_string())
    } else {
        state.title.clone()
    };
    state.title = Some(crate::load::save(title, &state.context)?);
    Ok(())
}

fn load(args: &[String], state: &mut AppState) -> Res<()> {
    let title = if let Some(arg) = single_opt_arg(args)? {
        arg.to_string()
    } else if let Some(title) = &state.title {
        title.clone()
    } else {
        state
            .input
            .prompt("Save title")
            .map_err(|e| e.to_string())?
    };

    state.context = crate::load::load(&title)?;
    state.title = Some(title);
    Ok(())
}

fn parse_command(input: &str) -> Res<(String, Vec<String>)> {
    enum ParseState {
        Dot,
        Command,
        Argument,
        Quote,
    }

    let mut state = ParseState::Dot;
    let mut command = String::new();
    let mut arg = String::new();
    let mut args = Vec::new();
    for char in input.chars() {
        match state {
            ParseState::Dot => match char {
                '.' => state = ParseState::Command,
                _ => return err("Command syntax: .<command> argument argument"),
            },
            ParseState::Command => match char {
                _ if char.is_alphabetic() => command.push(char),
                '"' => state = ParseState::Quote,
                ' ' => state = ParseState::Argument,
                _ => return Err(format!("Invalid character in command name: {char}")),
            },
            ParseState::Argument => match char {
                '"' => {
                    if !arg.is_empty() {
                        args.push(arg);
                        arg = String::new();
                    }
                    state = ParseState::Quote
                }
                ' ' => {
                    if !arg.is_empty() {
                        args.push(arg);
                        arg = String::new();
                    }
                }
                _ => arg.push(char),
            },
            ParseState::Quote => match char {
                '"' => {
                    state = ParseState::Argument;
                    args.push(arg);
                    arg = String::new();
                }
                _ => arg.push(char),
            },
        }
    }
    if !arg.is_empty() {
        if matches!(state, ParseState::Quote) {
            return err("Unmatched quote.");
        }
        args.push(arg);
    }
    Ok((command, args))
}

pub fn handle(input: &str, state: &mut AppState) -> Res<()> {
    let (command, args) = parse_command(input)?;
    for (name, func) in COMMANDS {
        if *name == command {
            return (func)(&args, state);
        }
    }
    Err(format!("Not a command: {command}"))
}

#[cfg(test)]
mod test {
    use crate::commands::parse_command;

    #[test]
    fn test_parse_command() {
        assert_eq!(parse_command(".exit").unwrap(), ("exit".into(), Vec::new()));
        assert_eq!(
            parse_command(".load name").unwrap(),
            ("load".into(), vec!["name".into()])
        );
        assert_eq!(
            parse_command(".set var \"quoted\"").unwrap(),
            ("set".into(), vec!["var".into(), "quoted".into()])
        );
        assert!(parse_command(".cmd \"unclosed quote").is_err());
        assert!(parse_command("cmd arg arg").is_err());
    }
}

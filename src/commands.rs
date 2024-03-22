use crate::{err, Res};

const COMMANDS: &[(&str, &'static dyn Fn(&[String]) -> Res<()>)] =
    &[("exit", &exit), ("save", &save), ("load", &load)];

pub fn exit(args: &[String]) -> Res<()> {
    const NOSAVE_ARGS: &[&str] = &["!", "nosave"];
    const USAGE: &str = "Usage: exit [nosave]";

    let skip_save = match args.len() {
        0 => false,
        1 => {
            let arg = args.first().unwrap().as_str();
            if NOSAVE_ARGS.contains(&arg) {
                true
            } else {
                return err(USAGE);
            }
        }
        _ => return err(USAGE),
    };

    if !skip_save {
        save(&[])?;
    }

    std::process::exit(0);
}

fn save(args: &[String]) -> Res<()> {
    err("Failed")
}

fn load(args: &[String]) -> Res<()> {
    err("Failed")
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

pub fn handle(input: &str) -> Res<()> {
    let (command, args) = parse_command(input)?;
    for (name, func) in COMMANDS {
        if *name == command {
            return (func)(&args);
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

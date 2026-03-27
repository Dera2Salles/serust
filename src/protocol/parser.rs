#[derive(Debug, PartialEq)]
pub enum Command {
    Login { username: String, password: String },
    List,
    Upload { filename: String, size: usize },
    Download { filename: String },
    Quit,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidCommand,
    MissingArguments,
    InvalidSize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidCommand => write!(f, "Commande invalide"),
            ParseError::MissingArguments => write!(f, "Arguments manquants"),
            ParseError::InvalidSize => write!(f, "Taille invalide"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return Err(ParseError::InvalidCommand);
    }

    match parts[0].to_uppercase().as_str() {
        "LOGIN" => {
            if parts.len() < 3 {
                return Err(ParseError::MissingArguments);
            }
            Ok(Command::Login {
                username: parts[1].to_string(),
                password: parts[2].to_string(),
            })
        }
        "LIST" => Ok(Command::List),
        "UPLOAD" => {
            if parts.len() < 3 {
                return Err(ParseError::MissingArguments);
            }
            let size = parts[2]
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidSize)?;

            Ok(Command::Upload {
                filename: parts[1].to_string(),
                size,
            })
        }
        "DOWNLOAD" => {
            if parts.len() < 2 {
                return Err(ParseError::MissingArguments);
            }
            Ok(Command::Download {
                filename: parts[1].to_string(),
            })
        }
        "QUIT" => Ok(Command::Quit),
        _ => Err(ParseError::InvalidCommand),
    }
}

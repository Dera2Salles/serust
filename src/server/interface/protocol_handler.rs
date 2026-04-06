use crate::common::error::DomainError;

#[derive(Debug, PartialEq)]
pub enum Command {
    Login { username: String, password: String },
    Upload { filename: String, size: u64 },
    Download { filename: String },
    List,
    Quit,
}

pub fn parse_command(line: &str) -> Result<Command, DomainError> {
    let line = line.trim();
    let parts: Vec<&str> = line.splitn(3, ' ').collect();

    match parts.as_slice() {
        ["LOGIN", username, password] => Ok(Command::Login {
            username: username.to_string(),
            password: password.to_string(),
        }),
        ["UPLOAD", filename, size_str] => {
            let size = size_str
                .parse::<u64>()
                .map_err(|_| DomainError::Internal("invalid_size".into()))?;
            Ok(Command::Upload {
                filename: filename.to_string(),
                size,
            })
        }
        ["DOWNLOAD", filename] => Ok(Command::Download {
            filename: filename.to_string(),
        }),
        ["LIST"] => Ok(Command::List),
        ["QUIT"] => Ok(Command::Quit),
        _ => Err(DomainError::Internal("invalid_command".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_login() {
        let cmd = parse_command("LOGIN alice secret").unwrap();
        assert_eq!(
            cmd,
            Command::Login {
                username: "alice".into(),
                password: "secret".into()
            }
        );
    }

    #[test]
    fn test_parse_upload() {
        let cmd = parse_command("UPLOAD photo.jpg 204800").unwrap();
        assert_eq!(
            cmd,
            Command::Upload {
                filename: "photo.jpg".into(),
                size: 204800
            }
        );
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse_command("LIST").unwrap(), Command::List);
    }

    #[test]
    fn test_parse_quit() {
        assert_eq!(parse_command("QUIT").unwrap(), Command::Quit);
    }

    #[test]
    fn test_invalid_command() {
        assert!(parse_command("FOOBAR").is_err());
    }
}

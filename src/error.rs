use std::fmt;

#[derive(Debug)]
pub enum BankToolsLibError {
    InvalidFormat(String),
    Io(std::io::Error),
}

impl std::fmt::Display for BankToolsLibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BankToolsLibError::InvalidFormat(format) => {
                write!(f, "ошибка чтения формата {format}: файл поврежден")
            }
            BankToolsLibError::Io(err) => write!(f, "ошибка ввода-вывода: {err}"),
        }
    }
}

impl std::error::Error for BankToolsLibError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // Указываем первопричину для системных ошибок
            BankToolsLibError::Io(err) => Some(err),
            _ => None,
        }
    }
}

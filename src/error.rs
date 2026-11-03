//! Типы ошибок библиотеки.

use std::fmt;
use std::io;

/// Ошибка библиотеки для работы с финансовыми операциями.
///
/// Объединяет ошибки ввода-вывода и ошибки разбора данных.
#[derive(Debug)]
pub enum BankError {
    /// Ошибка ввода-вывода.
    Io(io::Error),
    /// Синтаксическая ошибка входных данных (некорректное значение поля).
    Parse(String),
    /// Неизвестный или неподдерживаемый формат данных.
    UnknownFormat(String),
}

impl fmt::Display for BankError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BankError::Io(e) => write!(f, "ошибка ввода-вывода: {e}"),
            BankError::Parse(msg) => write!(f, "ошибка разбора: {msg}"),
            BankError::UnknownFormat(msg) => write!(f, "неизвестный формат: {msg}"),
        }
    }
}

impl std::error::Error for BankError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BankError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BankError {
    fn from(value: io::Error) -> Self {
        BankError::Io(value)
    }
}

use std::fmt;
use std::io;

use crate::format::DataFormat;

/// Ошибка библиотеки.
///
/// Объединяет ошибки ввода вывода, неизвестных данных
/// и разбора входных данных
#[derive(Debug)]
pub enum Error {
    /// Ошибка чтения и записи
    Io(std::io::Error),

    /// Не соответствует не одному поддерживаемому формату
    UnknownFormat(String),

    /// Входные данные не удалось разобрать.
    Parse {
        /// Формат, при разборе которого возникла ошибка.
        format: DataFormat,

        /// Номер строки для csv/text или номер записи для bin.
        location: Option<usize>,

        /// Поле, в котором найдена проблема.
        field: Option<&'static str>,

        /// Человекочитаемое описание проблемы.
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => {
                write!(f, "ошибка ввода-вывода: {err}")
            }

            Error::UnknownFormat(value) => {
                write!(
                    f,
                    "неизвестный формат: {value}; ожидается csv, text или bin"
                )
            }

            Error::Parse {
                format,
                location,
                field,
                message,
            } => {
                write!(f, "ошибка разбора формата {format}")?;

                if let Some(location) = location {
                    write!(f, ", позиция {location}")?;
                }

                if let Some(field) = field {
                    write!(f, ", поле {field}")?;
                }

                write!(f, ": {message}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Parse { .. } | Error::UnknownFormat(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::DataFormat;
    use std::error::Error as StdError;
    use std::io;

    #[test]
    fn converts_io_error() {
        let error: Error = io::Error::new(io::ErrorKind::NotFound, "файл не найден").into();

        assert!(matches!(error, Error::Io(_)));
    }

    #[test]
    fn displays_io_error() {
        let error = Error::from(io::Error::new(io::ErrorKind::NotFound, "файл не найден"));

        assert_eq!(error.to_string(), "ошибка ввода-вывода: файл не найден");
    }

    #[test]
    fn displays_unknown_format() {
        let error = Error::UnknownFormat("json".to_string());

        assert_eq!(
            error.to_string(),
            "неизвестный формат: json; ожидается csv, text или bin"
        );
    }

    #[test]
    fn displays_parse_error_with_full_context() {
        let error = Error::Parse {
            format: DataFormat::Csv,
            location: Some(3),
            field: Some("amount"),
            message: "ожидалось целое число".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "ошибка разбора формата csv, позиция 3, поле amount: ожидалось целое число"
        );
    }

    #[test]
    fn displays_parse_error_without_optional_context() {
        let error = Error::Parse {
            format: DataFormat::Bin,
            location: None,
            field: None,
            message: "неверный заголовок".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "ошибка разбора формата bin: неверный заголовок"
        );
    }

    #[test]
    fn io_error_is_exposed_as_source() {
        let error = Error::from(io::Error::new(io::ErrorKind::Other, "диск недоступен"));

        let source = error
            .source()
            .expect("у ошибки ввода-вывода должен быть source");

        assert_eq!(source.to_string(), "диск недоступен");
    }

    #[test]
    fn parse_error_has_no_source() {
        let error = Error::Parse {
            format: DataFormat::Text,
            location: None,
            field: None,
            message: "плохая строка".to_string(),
        };

        assert!(error.source().is_none());
    }
}

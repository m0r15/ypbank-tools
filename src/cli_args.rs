//! Общие помощники для разбора аргументов командной строки.

use std::fs::File;
use std::io::{self, Read};

use crate::error::BankError;

/// Возвращает значение, следующее за флагом `flag`.
///
/// Возвращает [`BankError::Parse`], если значение отсутствует или похоже
/// на следующий флаг (начинается с `--`).
pub fn next_value<I>(args: &mut I, flag: &str) -> Result<String, BankError>
where
    I: Iterator<Item = String>,
{
    match args.next() {
        Some(value) if !value.starts_with("--") => Ok(value),
        _ => Err(BankError::Parse(format!("флаг {flag} требует значение"))),
    }
}

/// Открывает входной поток: файл `path` или стандартный ввод, если `None`.
pub fn open_input(path: Option<&str>) -> Result<Box<dyn Read>, BankError> {
    match path {
        Some(p) => Ok(Box::new(File::open(p)?)),
        None => Ok(Box::new(io::stdin())),
    }
}

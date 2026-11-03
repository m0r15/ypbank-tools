//! Приложение `comparer`: сравнение двух файлов с транзакциями.

use std::env;
use std::fs::File;
use std::process::ExitCode;

use ypbank_tools::cli_args::next_value;
use ypbank_tools::{BankError, CompareResult, Format, compare, read_transactions};

/// Опции сравнения.
struct Config {
    /// Путь к левому файлу.
    left: String,
    /// Путь к правому файлу.
    right: String,
    /// Формат левого файла.
    left_format: Format,
    /// Формат правого файла.
    right_format: Format,
}

fn main() -> ExitCode {
    match run() {
        Ok(result) => match result {
            CompareResult::Equal => {
                println!("{result}");
                ExitCode::SUCCESS
            }
            _ => {
                println!("{result}");
                ExitCode::from(1)
            }
        },
        Err(e) => {
            eprintln!("comparer: {e}");
            eprintln!(
                "Использование: comparer [--left-format F] [--right-format F] <left> <right>"
            );
            eprintln!(
                "          или comparer --left <f> --right <f> [--left-format F] [--right-format F]"
            );
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<CompareResult, BankError> {
    let config = parse_args(env::args().skip(1))?;

    let left = read_transactions(File::open(&config.left)?, config.left_format)?;
    let right = read_transactions(File::open(&config.right)?, config.right_format)?;

    Ok(compare(&left, &right))
}

/// Разбирает аргументы командной строки.
fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Config, BankError> {
    let mut left = None;
    let mut right = None;
    let mut left_format = None;
    let mut right_format = None;
    let mut positional = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--left" => left = Some(next_value(&mut args, "--left")?),
            "--right" => right = Some(next_value(&mut args, "--right")?),
            "--left-format" => {
                left_format = Some(Format::parse(&next_value(&mut args, "--left-format")?)?);
            }
            "--right-format" => {
                right_format = Some(Format::parse(&next_value(&mut args, "--right-format")?)?);
            }
            other if other.starts_with("--") => {
                return Err(BankError::Parse(format!("неизвестный аргумент: {other}")));
            }
            other => positional.push(other.to_string()),
        }
    }

    // `--left`/`--right` и позиционные пути — два взаимоисключающих способа.
    let (left, right) = match (left, right) {
        (Some(l), Some(r)) => {
            if !positional.is_empty() {
                return Err(BankError::Parse(
                    "нельзя смешивать --left/--right и позиционные пути".to_string(),
                ));
            }
            (l, r)
        }
        (Some(_), None) => {
            return Err(BankError::Parse("задан --left без --right".to_string()));
        }
        (None, Some(_)) => {
            return Err(BankError::Parse("задан --right без --left".to_string()));
        }
        (None, None) => {
            if positional.len() != 2 {
                return Err(BankError::Parse(format!(
                    "ожидалось два файла (--left и --right либо два позиционных пути), получено {}",
                    positional.len()
                )));
            }
            (positional[0].clone(), positional[1].clone())
        }
    };

    // Если формат не задан явно, определяем его по расширению файла.
    let left_format = match left_format {
        Some(f) => f,
        None => Format::from_extension(&left)?,
    };
    let right_format = match right_format {
        Some(f) => f,
        None => Format::from_extension(&right)?,
    };

    Ok(Config {
        left,
        right,
        left_format,
        right_format,
    })
}

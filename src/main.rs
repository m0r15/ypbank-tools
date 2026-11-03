//! Приложение `converter`: конвертация данных между форматами.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use ypbank_tools::cli_args::{next_value, open_input};
use ypbank_tools::{BankError, Format, convert};

/// Опции конвертера.
struct Config {
    /// Входной формат.
    from: Format,
    /// Выходной формат.
    to: Format,
    /// Путь к входному файлу (`None` — читать stdin).
    input: Option<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("converter: {e}");
            eprintln!("Использование: converter --from <fmt> --to <fmt> [--input <path>]");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), BankError> {
    let config = parse_args(env::args().skip(1))?;

    let input = open_input(config.input.as_deref())?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    convert(input, config.from, &mut out, config.to)?;
    out.flush()?;
    Ok(())
}

/// Разбирает аргументы командной строки.
fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Result<Config, BankError> {
    let mut from = None;
    let mut to = None;
    let mut input = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => from = Some(Format::parse(&next_value(&mut args, "--from")?)?),
            "--to" => to = Some(Format::parse(&next_value(&mut args, "--to")?)?),
            "--input" => input = Some(next_value(&mut args, "--input")?),
            other => return Err(BankError::Parse(format!("неизвестный аргумент: {other}"))),
        }
    }

    let from = from.ok_or_else(|| BankError::Parse("не задан --from".to_string()))?;
    let to = to.ok_or_else(|| BankError::Parse("не задан --to".to_string()))?;

    Ok(Config { from, to, input })
}

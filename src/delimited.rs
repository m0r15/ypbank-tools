//! Общая логика чтения/записи форматов с разделителями (csv и text).

use std::io::{BufRead, BufReader, Read, Write};

use crate::error::BankError;
use crate::model::{Amount, Date, Kind, Transaction};

/// Читает транзакции из источника с разделителем.
///
/// Если задан `header`, первая строка проверяется на точное совпадение с ним.
/// Пустые строки пропускаются. Некорректная строка возвращает ошибку.
pub fn read<R: Read>(
    reader: R,
    delimiter: char,
    header: Option<&str>,
) -> Result<Vec<Transaction>, BankError> {
    let mut reader = BufReader::new(reader);

    if let Some(expected) = header {
        let mut first = String::new();
        reader.read_line(&mut first)?;
        if trim_line_end(&first) != expected {
            return Err(BankError::Parse(format!(
                "некорректный заголовок: ожидался {expected:?}, получен {:?}",
                trim_line_end(&first)
            )));
        }
    }

    let mut txs = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = trim_line_end(&line);
        if trimmed.is_empty() {
            continue;
        }
        txs.push(parse_line(trimmed, delimiter)?);
    }
    Ok(txs)
}

/// Записывает транзакции с разделителем и опциональным заголовком.
pub fn write<W: Write>(
    mut writer: W,
    txs: &[Transaction],
    delimiter: char,
    header: Option<&str>,
) -> Result<(), BankError> {
    if let Some(h) = header {
        writeln!(writer, "{h}")?;
    }
    for tx in txs {
        write_line(&mut writer, tx, delimiter)?;
    }
    Ok(())
}

/// Разбирает одну строку с разделителем в транзакцию.
fn parse_line(line: &str, delimiter: char) -> Result<Transaction, BankError> {
    let fields: Vec<&str> = line.split(delimiter).collect();
    if fields.len() != 4 {
        return Err(BankError::Parse(format!(
            "ожидалось 4 поля, получено {}: {line:?}",
            fields.len()
        )));
    }

    let date = Date::parse(fields[0].trim())?;
    let category = fields[1].trim();
    if category.is_empty() {
        return Err(BankError::Parse("пустая категория".to_string()));
    }
    let kind = Kind::parse(fields[2].trim())?;
    let amount = Amount::parse(fields[3].trim())?;

    Ok(Transaction {
        date,
        category: category.to_string(),
        kind,
        amount,
    })
}

/// Записывает одну транзакцию как строку с разделителем.
fn write_line<W: Write>(
    writer: &mut W,
    tx: &Transaction,
    delimiter: char,
) -> Result<(), BankError> {
    validate_category(&tx.category, delimiter)?;
    writeln!(
        writer,
        "{}{delimiter}{}{delimiter}{}{delimiter}{}",
        tx.date, tx.category, tx.kind, tx.amount
    )?;
    Ok(())
}

/// Проверяет, что категория не содержит разделитель и перевод строки.
fn validate_category(category: &str, delimiter: char) -> Result<(), BankError> {
    if category.contains(delimiter) || category.contains(['\n', '\r']) {
        return Err(BankError::Parse(format!(
            "категория не должна содержать разделитель или перевод строки: {category:?}"
        )));
    }
    Ok(())
}

/// Обрезает перевод строки (LF и/или CR) в конце строки.
fn trim_line_end(s: &str) -> &str {
    s.trim_end_matches(['\n', '\r'])
}

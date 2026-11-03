//! Форматы данных и общее API чтения/записи.

use std::fmt;
use std::io::{Read, Write};

use crate::bin_format;
use crate::csv_format;
use crate::error::BankError;
use crate::model::Transaction;
use crate::text_format;

/// Поддерживаемые форматы данных.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// CSV с заголовком `date,category,kind,amount`.
    Csv,
    /// Текстовый формат с разделителем `;` без заголовка.
    Text,
    /// Бинарный формат (схема в `README.md`).
    Bin,
}

impl Format {
    /// Разбирает формат из строки (`"csv"`, `"text"`/`"txt"`, `"bin"`).
    ///
    /// Возвращает [`BankError::UnknownFormat`], если строка неизвестна.
    pub fn parse(s: &str) -> Result<Self, BankError> {
        match s {
            "csv" => Ok(Format::Csv),
            "text" | "txt" => Ok(Format::Text),
            "bin" => Ok(Format::Bin),
            _ => Err(BankError::UnknownFormat(s.to_string())),
        }
    }

    /// Определяет формат по расширению имени файла.
    ///
    /// `txt`/`text` → text, `csv` → csv, `bin` → bin. Отсутствие расширения
    /// считается бинарным форматом. Возвращает [`BankError::UnknownFormat`],
    /// если расширение неизвестно.
    pub fn from_extension(path: &str) -> Result<Self, BankError> {
        let file_name = path.rsplit(['/', '\\']).next().unwrap_or(path);
        let ext = match file_name.rfind('.') {
            Some(idx) => file_name[idx + 1..].to_lowercase(),
            None => String::new(),
        };
        match ext.as_str() {
            "csv" => Ok(Format::Csv),
            "txt" | "text" => Ok(Format::Text),
            "bin" => Ok(Format::Bin),
            "" => Ok(Format::Bin),
            _ => Err(BankError::UnknownFormat(format!(
                "не удалось определить формат по расширению: {path:?}"
            ))),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Csv => f.write_str("csv"),
            Format::Text => f.write_str("text"),
            Format::Bin => f.write_str("bin"),
        }
    }
}

/// Читает список транзакций из источника `reader` в формате `format`.
pub fn read_transactions<R: Read>(
    reader: R,
    format: Format,
) -> Result<Vec<Transaction>, BankError> {
    match format {
        Format::Csv => csv_format::read(reader),
        Format::Text => text_format::read(reader),
        Format::Bin => bin_format::read(reader),
    }
}

/// Записывает список транзакций в приёмник `writer` в формате `format`.
pub fn write_transactions<W: Write>(
    writer: W,
    format: Format,
    txs: &[Transaction],
) -> Result<(), BankError> {
    match format {
        Format::Csv => csv_format::write(writer, txs),
        Format::Text => text_format::write(writer, txs),
        Format::Bin => bin_format::write(writer, txs),
    }
}

/// Конвертирует данные из формата `from` в формат `to`.
///
/// Данные читаются в общее представление транзакций, затем записываются
/// в целевой формат.
pub fn convert<R: Read, W: Write>(
    reader: R,
    from: Format,
    mut writer: W,
    to: Format,
) -> Result<(), BankError> {
    let txs = read_transactions(reader, from)?;
    write_transactions(&mut writer, to, &txs)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{Format, convert, read_transactions};
    use crate::model::Amount;

    #[test]
    fn converts_between_formats_via_cursor() {
        let csv = "date,category,kind,amount\n\
                   2026-04-01,salary,income,120000\n\
                   2026-04-02,rent,expense,40000\n";
        let mut text = Vec::new();
        convert(
            Cursor::new(csv.as_bytes()),
            Format::Csv,
            &mut text,
            Format::Text,
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(text).unwrap(),
            "2026-04-01;salary;income;120000\n2026-04-02;rent;expense;40000\n"
        );
    }

    #[test]
    fn reads_via_cursor() {
        let text = "2026-04-01;salary;income;120000\n";
        let txs = read_transactions(Cursor::new(text.as_bytes()), Format::Text).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].amount, Amount::new(120000));
    }

    #[test]
    fn round_trips_all_formats() {
        let original = "date,category,kind,amount\n2026-04-01,salary,income,120000\n2026-04-02,rent,expense,40000\n";

        let mut text = Vec::new();
        convert(
            Cursor::new(original.as_bytes()),
            Format::Csv,
            &mut text,
            Format::Text,
        )
        .unwrap();

        let mut bin = Vec::new();
        convert(Cursor::new(text), Format::Text, &mut bin, Format::Bin).unwrap();

        let mut back = Vec::new();
        convert(Cursor::new(bin), Format::Bin, &mut back, Format::Csv).unwrap();

        assert_eq!(String::from_utf8(back).unwrap(), original);
    }

    #[test]
    fn detects_format_by_extension() {
        assert_eq!(Format::from_extension("data.txt").unwrap(), Format::Text);
        assert_eq!(Format::from_extension("data.csv").unwrap(), Format::Csv);
        assert_eq!(Format::from_extension("data.bin").unwrap(), Format::Bin);
        assert_eq!(Format::from_extension("data").unwrap(), Format::Bin);
        assert!(Format::from_extension("data.json").is_err());

        // Точки в каталогах не должны влиять на определение расширения.
        assert_eq!(
            Format::from_extension("/data/v1.0/file.csv").unwrap(),
            Format::Csv
        );
        assert_eq!(
            Format::from_extension("/backups/2026.04/export").unwrap(),
            Format::Bin
        );
    }
}

//! Бинарный формат транзакций.
//!
//! Схема формата описана в `README.md`. Все числа little-endian, без padding.
//! Заголовок — 9 байт: magic `YPBN` (4), версия `1` (`u8`), количество
//! записей (`u32`). Запись: год `u16`, месяц `u8`, день `u8`, тип `u8`
//! (`0` — income, `1` — expense), сумма `i64`, длина категории `u32` и
//! UTF-8-байты категории. Максимальная длина категории — 1 МиБ,
//! максимальное количество записей — 1 000 000. После объявленного числа
//! записей лишние байты запрещены.

use std::io::{Read, Write};

use crate::error::BankError;
use crate::model::{Amount, Date, Kind, Transaction};

/// Сигнатура файла.
const MAGIC: &[u8; 4] = b"YPBN";
/// Версия формата.
const VERSION: u8 = 1;
/// Длина заголовка в байтах.
const HEADER_LEN: usize = 9;
/// Максимальная длина категории в байтах.
const MAX_CATEGORY_LEN: u32 = 1024 * 1024;
/// Максимальное количество записей в файле.
const MAX_COUNT: u32 = 1_000_000;

/// Читает транзакции из бинарного формата.
pub fn read<R: Read>(mut reader: R) -> Result<Vec<Transaction>, BankError> {
    let mut header = [0u8; HEADER_LEN];
    read_exact(&mut reader, &mut header, "заголовок")?;

    if &header[0..4] != MAGIC {
        return Err(BankError::Parse(
            "неверная сигнатура бинарного файла".to_string(),
        ));
    }
    if header[4] != VERSION {
        return Err(BankError::Parse(format!(
            "неподдерживаемая версия бинарного формата: {}",
            header[4]
        )));
    }
    let count = u32::from_le_bytes([header[5], header[6], header[7], header[8]]);
    if count > MAX_COUNT {
        return Err(BankError::Parse(format!("слишком много записей: {count}")));
    }

    let mut txs = Vec::new();
    for _ in 0..count as usize {
        txs.push(read_record(&mut reader)?);
    }

    // После объявленного числа записей лишние байты запрещены.
    let mut extra = [0u8; 1];
    match reader.read(&mut extra)? {
        0 => {}
        _ => {
            return Err(BankError::Parse(
                "лишние байты после последней записи".to_string(),
            ));
        }
    }

    Ok(txs)
}

/// Записывает транзакции в бинарный формат.
pub fn write<W: Write>(mut writer: W, txs: &[Transaction]) -> Result<(), BankError> {
    let count: u32 = txs
        .len()
        .try_into()
        .map_err(|_| BankError::Parse(format!("слишком много записей: {}", txs.len())))?;

    writer.write_all(MAGIC)?;
    writer.write_all(&[VERSION])?;
    writer.write_all(&count.to_le_bytes())?;

    for tx in txs {
        write_record(&mut writer, tx)?;
    }
    Ok(())
}

/// Читает одну запись из бинарного формата.
fn read_record<R: Read>(reader: &mut R) -> Result<Transaction, BankError> {
    let year = read_u16(reader, "год")?;
    let month = read_u8(reader, "месяц")?;
    let day = read_u8(reader, "день")?;
    let kind_byte = read_u8(reader, "тип операции")?;
    let amount = read_i64(reader, "сумма")?;
    let category_len = read_u32(reader, "длина категории")?;

    if category_len > MAX_CATEGORY_LEN {
        return Err(BankError::Parse(format!(
            "слишком длинная категория: {category_len} байт"
        )));
    }

    let mut buf = vec![0u8; category_len as usize];
    read_exact(reader, &mut buf, "категория")?;
    let category = String::from_utf8(buf)
        .map_err(|e| BankError::Parse(format!("категория не является UTF-8: {e}")))?;

    let date = Date::new(year, month, day)?;
    let kind = match kind_byte {
        0 => Kind::Income,
        1 => Kind::Expense,
        other => {
            return Err(BankError::Parse(format!(
                "неизвестный тип операции: {other}"
            )));
        }
    };

    Ok(Transaction {
        date,
        category,
        kind,
        amount: Amount::new(amount),
    })
}

/// Записывает одну запись в бинарный формат.
fn write_record<W: Write>(writer: &mut W, tx: &Transaction) -> Result<(), BankError> {
    let category_len: u32 = tx.category.len().try_into().map_err(|_| {
        BankError::Parse(format!("слишком длинная категория: {}", tx.category.len()))
    })?;
    if category_len > MAX_CATEGORY_LEN {
        return Err(BankError::Parse(format!(
            "слишком длинная категория: {category_len} байт"
        )));
    }

    writer.write_all(&tx.date.year().to_le_bytes())?;
    writer.write_all(&[tx.date.month()])?;
    writer.write_all(&[tx.date.day()])?;
    let kind_byte = match tx.kind {
        Kind::Income => 0u8,
        Kind::Expense => 1u8,
    };
    writer.write_all(&[kind_byte])?;
    writer.write_all(&tx.amount.value().to_le_bytes())?;
    writer.write_all(&category_len.to_le_bytes())?;
    writer.write_all(tx.category.as_bytes())?;
    Ok(())
}

/// Читает `u8` из источника.
fn read_u8<R: Read>(reader: &mut R, what: &str) -> Result<u8, BankError> {
    let mut buf = [0u8; 1];
    read_exact(reader, &mut buf, what)?;
    Ok(buf[0])
}

/// Читает `u16` little-endian из источника.
fn read_u16<R: Read>(reader: &mut R, what: &str) -> Result<u16, BankError> {
    let mut buf = [0u8; 2];
    read_exact(reader, &mut buf, what)?;
    Ok(u16::from_le_bytes(buf))
}

/// Читает `u32` little-endian из источника.
fn read_u32<R: Read>(reader: &mut R, what: &str) -> Result<u32, BankError> {
    let mut buf = [0u8; 4];
    read_exact(reader, &mut buf, what)?;
    Ok(u32::from_le_bytes(buf))
}

/// Читает `i64` little-endian из источника.
fn read_i64<R: Read>(reader: &mut R, what: &str) -> Result<i64, BankError> {
    let mut buf = [0u8; 8];
    read_exact(reader, &mut buf, what)?;
    Ok(i64::from_le_bytes(buf))
}

/// Читает ровно `buf.len()` байт, преобразуя обрезание в понятную ошибку.
fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8], what: &str) -> Result<(), BankError> {
    reader.read_exact(buf).map_err(|e| match e.kind() {
        std::io::ErrorKind::UnexpectedEof => {
            BankError::Parse(format!("бинарный файл обрезан при чтении {what}"))
        }
        _ => BankError::Io(e),
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{read, write};
    use crate::error::BankError;
    use crate::model::{Amount, Date, Kind, Transaction};

    fn sample() -> Vec<Transaction> {
        vec![
            Transaction {
                date: Date::new(2026, 4, 1).unwrap(),
                category: "salary".to_string(),
                kind: Kind::Income,
                amount: Amount::new(120000),
            },
            Transaction {
                date: Date::new(2026, 4, 2).unwrap(),
                category: "rent".to_string(),
                kind: Kind::Expense,
                amount: Amount::new(40000),
            },
        ]
    }

    #[test]
    fn round_trips_bin() {
        let txs = sample();
        let mut buf = Vec::new();
        write(&mut buf, &txs).unwrap();
        let parsed = read(Cursor::new(buf)).unwrap();
        assert_eq!(parsed, txs);
    }

    #[test]
    fn rejects_truncated_bin() {
        let txs = sample();
        let mut buf = Vec::new();
        write(&mut buf, &txs).unwrap();
        let truncated = &buf[..buf.len() - 3];
        assert!(read(Cursor::new(truncated)).is_err());
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = b"XXXX".to_vec();
        buf.extend_from_slice(&[1, 0, 0, 0, 0]);
        assert!(read(Cursor::new(buf)).is_err());
    }

    #[test]
    fn rejects_too_many_records() {
        // Заголовок: magic + версия + количество записей > MAX_COUNT.
        let mut buf = b"YPBN".to_vec();
        buf.push(1);
        buf.extend_from_slice(&u32::MAX.to_le_bytes());
        match read(Cursor::new(buf)) {
            Err(BankError::Parse(msg)) => assert!(msg.contains("слишком много записей")),
            other => panic!("ожидалась ошибка разбора, получено {other:?}"),
        }
    }
}

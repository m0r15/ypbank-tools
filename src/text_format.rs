//! Текстовый формат с разделителем `;`.

use std::io::{Read, Write};

use crate::delimited;
use crate::error::BankError;
use crate::model::Transaction;

/// Разделитель полей text.
const DELIMITER: char = ';';

/// Читает транзакции из text (без заголовка).
pub fn read<R: Read>(reader: R) -> Result<Vec<Transaction>, BankError> {
    delimited::read(reader, DELIMITER, None)
}

/// Записывает транзакции в text (без заголовка).
pub fn write<W: Write>(writer: W, txs: &[Transaction]) -> Result<(), BankError> {
    delimited::write(writer, txs, DELIMITER, None)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::read;
    use crate::model::{Amount, Date, Kind};

    #[test]
    fn parses_valid_text() {
        let data = "2026-04-01;salary;income;120000\n2026-04-02;rent;expense;40000\n";
        let txs = read(Cursor::new(data)).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].date, Date::new(2026, 4, 1).unwrap());
        assert_eq!(txs[0].category, "salary");
        assert_eq!(txs[0].kind, Kind::Income);
        assert_eq!(txs[0].amount, Amount::new(120000));
        assert_eq!(txs[1].kind, Kind::Expense);
    }

    #[test]
    fn rejects_invalid_amount() {
        let data = "2026-04-01;salary;income;not_a_number\n";
        assert!(read(Cursor::new(data)).is_err());
    }
}

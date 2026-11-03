//! Формат CSV.

use std::io::{Read, Write};

use crate::delimited;
use crate::error::BankError;
use crate::model::Transaction;

/// Разделитель полей CSV.
const DELIMITER: char = ',';

/// Заголовок CSV.
const HEADER: &str = "date,category,kind,amount";

/// Читает транзакции из CSV (первая строка — заголовок).
pub fn read<R: Read>(reader: R) -> Result<Vec<Transaction>, BankError> {
    delimited::read(reader, DELIMITER, Some(HEADER))
}

/// Записывает транзакции в CSV (с заголовком).
pub fn write<W: Write>(writer: W, txs: &[Transaction]) -> Result<(), BankError> {
    delimited::write(writer, txs, DELIMITER, Some(HEADER))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::read;
    use crate::model::{Amount, Date, Kind};

    #[test]
    fn parses_valid_csv() {
        let data = "date,category,kind,amount\n\
                    2026-04-01,salary,income,120000\n\
                    2026-04-02,rent,expense,40000\n";
        let txs = read(Cursor::new(data)).unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(txs[0].date, Date::new(2026, 4, 1).unwrap());
        assert_eq!(txs[0].category, "salary");
        assert_eq!(txs[0].kind, Kind::Income);
        assert_eq!(txs[0].amount, Amount::new(120000));
        assert_eq!(txs[1].kind, Kind::Expense);
        assert_eq!(txs[1].amount, Amount::new(40000));
    }

    #[test]
    fn rejects_unknown_kind() {
        let data = "date,category,kind,amount\n2026-04-01,salary,bonus,120000\n";
        assert!(read(Cursor::new(data)).is_err());
    }

    #[test]
    fn rejects_wrong_header() {
        let data = "foo,bar,baz,qux\n2026-04-01,salary,income,120000\n";
        assert!(read(Cursor::new(data)).is_err());
    }
}

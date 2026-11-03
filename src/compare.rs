//! Сравнение списков транзакций.

use std::fmt;

use crate::model::Transaction;

/// Поле транзакции, в котором найдено различие.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareField {
    /// Дата.
    Date,
    /// Категория.
    Category,
    /// Тип операции.
    Kind,
    /// Сумма.
    Amount,
}

impl fmt::Display for CompareField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareField::Date => f.write_str("date"),
            CompareField::Category => f.write_str("category"),
            CompareField::Kind => f.write_str("kind"),
            CompareField::Amount => f.write_str("amount"),
        }
    }
}

/// Результат сравнения двух списков транзакций.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareResult {
    /// Списки полностью совпадают (по количеству и по каждой записи).
    Equal,
    /// Количество записей различается.
    LengthMismatch {
        /// Число записей в левом списке.
        left: usize,
        /// Число записей в правом списке.
        right: usize,
    },
    /// Записи отличаются на позиции `position` (с 1) в поле `field`.
    Different {
        /// Позиция первого расхождения, начиная с 1.
        position: usize,
        /// Поле, в котором найдено различие.
        field: CompareField,
        /// Значение из левого списка.
        left: String,
        /// Значение из правого списка.
        right: String,
    },
}

impl fmt::Display for CompareResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareResult::Equal => f.write_str("Файлы совпадают"),
            CompareResult::LengthMismatch { left, right } => {
                write!(f, "Количество транзакций различается: {left} != {right}")
            }
            CompareResult::Different {
                position,
                field,
                left,
                right,
            } => write!(
                f,
                "Транзакции отличаются на позиции {position}: {field}: {left} != {right}"
            ),
        }
    }
}

/// Сравнивает два списка транзакций поэлементно.
///
/// Порядок и количество записей важны. Сравнение идёт от первой записи к
/// последней и останавливается на первом различии. Возвращает
/// [`CompareResult::Equal`] при полном совпадении.
pub fn compare(left: &[Transaction], right: &[Transaction]) -> CompareResult {
    if left.len() != right.len() {
        return CompareResult::LengthMismatch {
            left: left.len(),
            right: right.len(),
        };
    }

    for (index, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        if l.date != r.date {
            return CompareResult::Different {
                position: index + 1,
                field: CompareField::Date,
                left: l.date.to_string(),
                right: r.date.to_string(),
            };
        }
        if l.category != r.category {
            return CompareResult::Different {
                position: index + 1,
                field: CompareField::Category,
                left: l.category.clone(),
                right: r.category.clone(),
            };
        }
        if l.kind != r.kind {
            return CompareResult::Different {
                position: index + 1,
                field: CompareField::Kind,
                left: l.kind.to_string(),
                right: r.kind.to_string(),
            };
        }
        if l.amount != r.amount {
            return CompareResult::Different {
                position: index + 1,
                field: CompareField::Amount,
                left: l.amount.to_string(),
                right: r.amount.to_string(),
            };
        }
    }

    CompareResult::Equal
}

#[cfg(test)]
mod tests {
    use super::{CompareField, CompareResult, compare};
    use crate::model::{Amount, Date, Kind, Transaction};

    fn tx(year: u16, month: u8, day: u8, category: &str, kind: Kind, amount: i64) -> Transaction {
        Transaction {
            date: Date::new(year, month, day).unwrap(),
            category: category.to_string(),
            kind,
            amount: Amount::new(amount),
        }
    }

    #[test]
    fn equal_lists_match() {
        let a = vec![tx(2026, 4, 1, "salary", Kind::Income, 120000)];
        let b = vec![tx(2026, 4, 1, "salary", Kind::Income, 120000)];
        assert_eq!(compare(&a, &b), CompareResult::Equal);
    }

    #[test]
    fn different_amount_reported() {
        let a = vec![tx(2026, 4, 1, "salary", Kind::Income, 120000)];
        let b = vec![tx(2026, 4, 1, "salary", Kind::Income, 130000)];
        match compare(&a, &b) {
            CompareResult::Different {
                position,
                field,
                left,
                right,
            } => {
                assert_eq!(position, 1);
                assert_eq!(field, CompareField::Amount);
                assert_eq!(left, "120000");
                assert_eq!(right, "130000");
            }
            other => panic!("ожидалось различие, получено {other:?}"),
        }
    }

    #[test]
    fn different_category_reported() {
        let a = vec![tx(2026, 4, 1, "salary", Kind::Income, 120000)];
        let b = vec![tx(2026, 4, 1, "bonus", Kind::Income, 120000)];
        match compare(&a, &b) {
            CompareResult::Different { field, .. } => {
                assert_eq!(field, CompareField::Category);
            }
            other => panic!("ожидалось различие, получено {other:?}"),
        }
    }

    #[test]
    fn length_mismatch_reported() {
        let a = vec![tx(2026, 4, 1, "salary", Kind::Income, 120000)];
        let b = vec![
            tx(2026, 4, 1, "salary", Kind::Income, 120000),
            tx(2026, 4, 2, "rent", Kind::Expense, 40000),
        ];
        assert_eq!(
            compare(&a, &b),
            CompareResult::LengthMismatch { left: 1, right: 2 }
        );
    }
}

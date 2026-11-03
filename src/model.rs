//! Доменная модель финансовых операций.

use std::fmt;

use crate::error::BankError;

/// Тип финансовой операции.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Поступление средств.
    Income,
    /// Расход средств.
    Expense,
}

impl Kind {
    /// Разбирает тип операции из строки `"income"` или `"expense"`.
    ///
    /// Возвращает [`BankError::Parse`], если значение неизвестно.
    pub fn parse(s: &str) -> Result<Self, BankError> {
        match s {
            "income" => Ok(Kind::Income),
            "expense" => Ok(Kind::Expense),
            _ => Err(BankError::Parse(format!("неизвестный тип операции: {s:?}"))),
        }
    }

    /// Строковое представление типа в форматах csv и text.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Income => "income",
            Kind::Expense => "expense",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Календарная дата: год, месяц и день.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// Создаёт дату, проверяя корректность месяца и дня.
    ///
    /// Возвращает [`BankError::Parse`], если месяц вне `1..=12` или день
    /// не существует в данном месяце (с учётом високосного года).
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, BankError> {
        if !(1..=12).contains(&month) {
            return Err(BankError::Parse(format!("некорректный месяц: {month}")));
        }
        if day < 1 || day > days_in_month(year, month) {
            return Err(BankError::Parse(format!(
                "некорректный день: {year:04}-{month:02}-{day:02}"
            )));
        }
        Ok(Date { year, month, day })
    }

    /// Разбирает дату из строки `YYYY-MM-DD`.
    ///
    /// Возвращает [`BankError::Parse`], если строка не соответствует формату
    /// или содержит некорректные значения.
    pub fn parse(s: &str) -> Result<Self, BankError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 || parts[0].len() != 4 {
            return Err(BankError::Parse(format!("некорректная дата: {s:?}")));
        }
        let year = parts[0]
            .parse::<u16>()
            .map_err(|_| BankError::Parse(format!("некорректная дата: {s:?}")))?;
        let month = parts[1]
            .parse::<u8>()
            .map_err(|_| BankError::Parse(format!("некорректная дата: {s:?}")))?;
        let day = parts[2]
            .parse::<u8>()
            .map_err(|_| BankError::Parse(format!("некорректная дата: {s:?}")))?;
        Date::new(year, month, day)
    }

    /// Возвращает год.
    pub fn year(self) -> u16 {
        self.year
    }

    /// Возвращает месяц.
    pub fn month(self) -> u8 {
        self.month
    }

    /// Возвращает день.
    pub fn day(self) -> u8 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Денежная сумма операции в целых единицах.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Amount(i64);

impl Amount {
    /// Создаёт сумму из целого числа.
    pub fn new(value: i64) -> Self {
        Amount(value)
    }

    /// Возвращает числовое значение суммы.
    pub fn value(self) -> i64 {
        self.0
    }

    /// Разбирает сумму из строки с целым числом.
    ///
    /// Возвращает [`BankError::Parse`], если строка не является целым числом.
    pub fn parse(s: &str) -> Result<Self, BankError> {
        let value = s
            .trim()
            .parse::<i64>()
            .map_err(|_| BankError::Parse(format!("некорректная сумма: {s:?}")))?;
        Ok(Amount(value))
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Финансовая операция.
///
/// Единое внутреннее представление транзакции, через которое проходят
/// и конвертация, и сравнение.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Transaction {
    /// Дата операции.
    pub date: Date,
    /// Категория операции.
    pub category: String,
    /// Тип операции.
    pub kind: Kind,
    /// Сумма операции.
    pub amount: Amount,
}

/// Проверяет, является ли год високосным.
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Возвращает число дней в месяце с учётом високосного года.
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[test]
    fn parses_canonical_date() {
        assert_eq!(
            Date::parse("2026-04-01").unwrap(),
            Date::new(2026, 4, 1).unwrap()
        );
    }

    #[test]
    fn rejects_short_year() {
        assert!(Date::parse("26-04-01").is_err());
    }

    #[test]
    fn rejects_invalid_date() {
        assert!(Date::parse("2026-13-01").is_err());
        assert!(Date::parse("2026-02-30").is_err());
    }
}

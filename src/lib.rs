//! Библиотека для работы с финансовыми операциями.
//!
//! Предоставляет единое представление транзакций, чтение и запись в форматах
//! csv, text и bin, а также конвертацию и сравнение списков транзакций.

pub mod cli_args;
pub mod compare;
pub mod error;
pub mod format;
pub mod model;

mod bin_format;
mod csv_format;
mod delimited;
mod text_format;

pub use compare::{CompareField, CompareResult, compare};
pub use error::BankError;
pub use format::{Format, convert, read_transactions, write_transactions};
pub use model::{Amount, Date, Kind, Transaction};

use std::fmt;

use crate::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum DataFormat {
    Csv,
    Bin,
    Text,
}

impl std::str::FromStr for DataFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "csv" => Ok(DataFormat::Csv),
            "bin" => Ok(DataFormat::Bin),
            "text" => Ok(DataFormat::Text),
            other => Err(Error::UnknownFormat(other.to_owned())),
        }
    }
}

impl fmt::Display for DataFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DataFormat::Bin => "bin",
            DataFormat::Csv => "csv",
            DataFormat::Text => "text",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::str::FromStr;

    #[test]
    fn parses_supported_formats() {
        assert_eq!(DataFormat::from_str("csv").unwrap(), DataFormat::Csv);
        assert_eq!(DataFormat::from_str("text").unwrap(), DataFormat::Text);
        assert_eq!(DataFormat::from_str("bin").unwrap(), DataFormat::Bin);
    }

    #[test]
    fn rejects_unknown_format() {
        let error = DataFormat::from_str("json").unwrap_err();

        assert!(matches!(
            error,
            Error::UnknownFormat(value) if value == "json"
        ));
    }

    #[test]
    fn displays_format() {
        assert_eq!(DataFormat::Csv.to_string(), "csv");
        assert_eq!(DataFormat::Text.to_string(), "text");
        assert_eq!(DataFormat::Bin.to_string(), "bin");
    }

    #[test]
    fn parse_supported_format_case_insensetive() {
        assert_eq!(DataFormat::from_str("CSV").unwrap(), DataFormat::Csv);
        assert_eq!(DataFormat::from_str("TEXT").unwrap(), DataFormat::Text);
        assert_eq!(DataFormat::from_str("BIN").unwrap(), DataFormat::Bin);
    }
}

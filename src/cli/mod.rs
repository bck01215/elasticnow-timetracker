pub mod args;
pub mod config;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_time_validator() {
        assert_eq!(args::range_format_validate("2010-01-01").unwrap(), ());
    }
}

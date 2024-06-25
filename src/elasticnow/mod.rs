pub mod elasticnow;
pub mod servicenow;
pub mod servicenow_structs;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_time_add_to_epoch_basic() {
        assert_eq!(
            servicenow::time_add_to_epoch("1h2m").unwrap(),
            "1970-01-01+01:02:00"
        );
    }

    #[test]
    fn test_time_add_to_epoch_too_many_hours() {
        assert_eq!(
            servicenow::time_add_to_epoch("20h0m")
                .unwrap_err()
                .to_string(),
            "Invalid time format. Values must be below 20 for hours, 60 for minutes"
        );
    }

    #[test]
    fn test_time_add_to_epoch_too_many_minutes() {
        assert_eq!(
            servicenow::time_add_to_epoch("0h60m")
                .unwrap_err()
                .to_string(),
            "Invalid time format. Values must be below 20 for hours, 60 for minutes"
        );
    }
    #[test]
    fn test_time_only_hour() {
        assert_eq!(
            servicenow::time_add_to_epoch("1h").unwrap(),
            "1970-01-01+01:00:00"
        );
    }

    #[test]
    fn test_time_only_minute() {
        assert_eq!(
            servicenow::time_add_to_epoch("1m").unwrap(),
            "1970-01-01+00:01:00"
        );
    }

    #[test]
    fn test_time_no_time() {
        assert_eq!(
            servicenow::time_add_to_epoch("0h").unwrap_err().to_string(),
            "Time worked must be greater than 0 minutes"
        );
    }
}

use serde::{Deserialize, Deserializer, de::Error};
use std::time::Duration;

pub(crate) fn deserialize_duration_optional<'de, D>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    match deserialize_duration(deserializer) {
        Ok(result) => Ok(Some(result)),
        Err(_) => Ok(None),
    }
}

pub(crate) fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let (hours_str, rest) = string
        .split_once(':')
        .ok_or(Error::missing_field("First :"))?;
    let (minutes_str, rest) = rest
        .split_once(':')
        .ok_or(Error::missing_field("Second :"))?;
    let (seconds_str, nanos_str) = rest.split_once('.').ok_or(Error::missing_field("."))?;

    let hours: u64 = hours_str.parse().map_err(|_| {
        Error::invalid_value(serde::de::Unexpected::Str("String value"), &"numeric value")
    })?;
    let minutes: u64 = minutes_str.parse().map_err(|_| {
        Error::invalid_value(serde::de::Unexpected::Str("String value"), &"numeric value")
    })?;
    let seconds: u64 = seconds_str.parse().map_err(|_| {
        Error::invalid_value(serde::de::Unexpected::Str("String value"), &"numeric value")
    })?;
    let nanos: u64 = nanos_str.parse().map_err(|_| {
        Error::invalid_value(serde::de::Unexpected::Str("String value"), &"numeric value")
    })?;

    Ok(Duration::from_hours(hours)
        + Duration::from_mins(minutes)
        + Duration::from_secs(seconds)
        + Duration::from_nanos(nanos))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct DurationTest(#[serde(deserialize_with = "super::deserialize_duration")] Duration);

    #[test]
    fn test_duration_parsing() {
        assert_eq!(
            DurationTest(Duration::from_hours(2)),
            serde_json::from_str(r#""02:00:00.0""#).unwrap()
        );

        assert_eq!(
            DurationTest(Duration::from_hours(2) + Duration::from_mins(38)),
            serde_json::from_str(r#""02:38:00.0""#).unwrap()
        );

        assert_eq!(
            DurationTest(Duration::from_millis(120)),
            serde_json::from_str(r#""00:00:00.120000000""#).unwrap()
        );
    }
}

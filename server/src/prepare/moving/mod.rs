mod movies;
mod series;

pub use movies::generate_movie_media;
pub use series::generate_series_media;

const URL_SAFE_NON_ALPHANUMERIC_CHARS: [char; 11] =
    ['$', '-', '_', '.', '+', '!', '*', '\'', '(', ')', ','];

fn sanitize_name_for_url(input: &str) -> String {
    input
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() | URL_SAFE_NON_ALPHANUMERIC_CHARS.contains(&char) {
                char
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::prepare::moving::sanitize_name_for_url;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_name_for_url("Hello World"), "Hello_World");
        assert_eq!(sanitize_name_for_url("valid"), "valid");
        assert_eq!(sanitize_name_for_url("|nvalid"), "_nvalid");
        assert_eq!(sanitize_name_for_url("bo$$"), "bo$$");
        assert_eq!(sanitize_name_for_url("Co!!"), "Co!!");
        assert_eq!(sanitize_name_for_url(":nvalid"), "_nvalid");
    }
}

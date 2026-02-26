//! The base64 create isn't intuitive enough to use directly, so we abstract it

use base64::{DecodeError, Engine as _, engine::general_purpose::URL_SAFE};

/// Encodes a string into a URL safe b64 string.
/// Handy when you need to use a name in a URL.
pub fn encode_url_safe(input: &str) -> String {
    URL_SAFE.encode(input)
}

/// Decodes the given base64 string into a...String!
pub fn decode_url_safe(decoded_input: &str) -> Result<String, DecodeError> {
    let bytes = URL_SAFE.decode(decoded_input)?;
    String::try_from(bytes).map_err(|_| DecodeError::InvalidByte(0, 0))
}

#[cfg(test)]
mod tests {
    use crate::encode_decode::{decode_url_safe, encode_url_safe};

    #[test]
    fn encode_and_decode() {
        assert_eq!(encode_url_safe("milk"), "bWlsaw==");
        assert_eq!(
            decode_url_safe(&encode_url_safe("cookies")).unwrap(),
            "cookies"
        );
        assert_eq!(
            decode_url_safe(&encode_url_safe("boo / %%!! scary stuff 2^#%^&*#@$@@ü")).unwrap(),
            "boo / %%!! scary stuff 2^#%^&*#@$@@ü"
        );
    }
}

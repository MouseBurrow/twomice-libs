use serde::Serialize;

const B62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn random_b62(len: usize) -> String {
    use rand::Rng;
    (0..len)
        .map(|_| {
            let idx = rand::thread_rng().gen_range(0..B62_CHARS.len());
            B62_CHARS[idx] as char
        })
        .collect()
}

pub fn encode_b62(value: i64) -> String {
    if value == 0 {
        return "0".to_string();
    }
    let mut positive = value.unsigned_abs();
    let mut chars = Vec::new();
    while positive > 0 {
        let idx = (positive % 62) as usize;
        chars.push(B62_CHARS[idx] as char);
        positive /= 62;
    }
    if value < 0 {
        chars.push('-');
    }
    chars.reverse();
    chars.into_iter().collect()
}

pub fn decode_b62(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }
    let (sign, digits) = if s.starts_with('-') {
        (-1i64, &s[1..])
    } else {
        (1, s)
    };
    if digits.is_empty() {
        return None;
    }
    let mut value: i64 = 0;
    for c in digits.chars() {
        let idx = B62_CHARS.iter().position(|&ch| ch == c as u8)?;
        value = value.checked_mul(62)?;
        value = value.checked_add(idx as i64)?;
    }
    if sign == -1 && value == 0 {
        return None;
    }
    Some(value * sign)
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            total: None,
            limit: None,
            offset: None,
        }
    }

    pub fn with_total(mut self, total: i64) -> Self {
        self.total = Some(total);
        self
    }

    pub fn with_pagination(mut self, limit: i64, offset: i64) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_b62_has_correct_length() {
        for len in [1, 5, 10, 32] {
            let s = random_b62(len);
            assert_eq!(s.len(), len, "length {len}");
        }
    }

    #[test]
    fn random_b62_uses_valid_chars() {
        let s = random_b62(1000);
        for c in s.chars() {
            assert!(c.is_ascii_alphanumeric(), "invalid char '{c}'");
        }
    }

    #[test]
    fn random_b62_produces_different_values() {
        let a = random_b62(10);
        let b = random_b62(10);
        assert_ne!(a, b);
    }

    #[test]
    fn encode_decode_roundtrip() {
        for val in [0, 1, 10, 61, 62, 100, 1000, 1234567890, i64::MAX] {
            let encoded = encode_b62(val);
            let decoded = decode_b62(&encoded).unwrap();
            assert_eq!(decoded, val, "roundtrip failed for {val}: encoded={encoded}");
        }
    }

    #[test]
    fn decode_rejects_empty() {
        assert!(decode_b62("").is_none());
    }

    #[test]
    fn decode_rejects_invalid_char() {
        assert!(decode_b62("hello!").is_none());
    }

    #[test]
    fn negative_roundtrip() {
        for val in [-1, -10, -62, -1000] {
            let encoded = encode_b62(val);
            let decoded = decode_b62(&encoded).unwrap();
            assert_eq!(decoded, val, "roundtrip failed for {val}: encoded={encoded}");
        }
    }

    #[test]
    fn paginated_response_builder() {
        let p = PaginatedResponse::new(vec![1, 2, 3])
            .with_total(10)
            .with_pagination(3, 0);
        assert_eq!(p.data, vec![1, 2, 3]);
        assert_eq!(p.total, Some(10));
        assert_eq!(p.limit, Some(3));
        assert_eq!(p.offset, Some(0));
    }
}

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

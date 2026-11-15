//! Locale-aware number formatting.

/// Separators of a locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFormat {
    pub decimal: char,
    pub group: char,
}

impl NumberFormat {
    pub fn int(&self, value: i64) -> String {
        let digits = value.unsigned_abs().to_string();
        let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
        if value < 0 {
            out.push('−');
        }
        for (i, c) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(self.group);
            }
            out.push(c);
        }
        out
    }

    pub fn decimal(&self, value: f64, digits: usize) -> String {
        let text = format!("{:.*}", digits, value.abs());
        let (int_part, frac) = text.split_once('.').unwrap_or((&text, ""));
        let int_value: i64 = int_part.parse().unwrap_or(0);
        let mut out = self.int(int_value);
        if value < 0.0 && text.chars().any(|c| c.is_ascii_digit() && c != '0') {
            out.insert(0, '−');
        }
        if !frac.is_empty() {
            out.push(self.decimal);
            out.push_str(frac);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::NumberFormat;

    const PT: NumberFormat = NumberFormat {
        decimal: ',',
        group: '.',
    };
    const EN: NumberFormat = NumberFormat {
        decimal: '.',
        group: ',',
    };

    #[test]
    fn groups_thousands() {
        assert_eq!(PT.int(1_234_567), "1.234.567");
        assert_eq!(EN.int(1_234_567), "1,234,567");
        assert_eq!(PT.int(999), "999");
        assert_eq!(PT.int(-1000), "−1.000");
        assert_eq!(PT.int(0), "0");
    }

    #[test]
    fn formats_decimals() {
        assert_eq!(PT.decimal(1234.567, 2), "1.234,57");
        assert_eq!(EN.decimal(0.5, 1), "0.5");
        assert_eq!(PT.decimal(-2.25, 1), "−2,2");
        assert_eq!(PT.decimal(-0.01, 1), "0,0");
    }
}

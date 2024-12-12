use std::cmp::Ordering;

/// Represents a bound in a range query
#[derive(Debug, Clone, PartialEq)]
pub enum RangeBound<T> {
    Included(T),
    Excluded(T),
    Unbounded,
}

/// Trait for types that support collation operations
pub trait Collation {
    /// Convert the value to its binary representation for collation
    fn to_bytes(&self) -> Vec<u8>;

    /// Returns the immediate successor's binary representation if one exists
    fn successor_bytes(&self) -> Option<Vec<u8>>;

    /// Returns the immediate predecessor's binary representation if one exists
    fn predecessor_bytes(&self) -> Option<Vec<u8>>;

    /// Returns true if this value represents a minimum bound in its domain
    fn is_minimum(&self) -> bool;

    /// Returns true if this value represents a maximum bound in its domain
    fn is_maximum(&self) -> bool;

    /// Compare two values in the collation order
    fn compare(&self, other: &Self) -> Ordering;

    fn is_in_range(&self, lower: RangeBound<&Self>, upper: RangeBound<&Self>) -> bool {
        match (lower, upper) {
            (RangeBound::Included(l), RangeBound::Included(u)) => {
                self.compare(l) != Ordering::Less && self.compare(u) != Ordering::Greater
            }
            (RangeBound::Included(l), RangeBound::Excluded(u)) => {
                self.compare(l) != Ordering::Less && self.compare(u) == Ordering::Less
            }
            (RangeBound::Excluded(l), RangeBound::Included(u)) => {
                self.compare(l) == Ordering::Greater && self.compare(u) != Ordering::Greater
            }
            (RangeBound::Excluded(l), RangeBound::Excluded(u)) => {
                self.compare(l) == Ordering::Greater && self.compare(u) == Ordering::Less
            }
            (RangeBound::Unbounded, RangeBound::Included(u)) => {
                self.compare(u) != Ordering::Greater
            }
            (RangeBound::Unbounded, RangeBound::Excluded(u)) => self.compare(u) == Ordering::Less,
            (RangeBound::Included(l), RangeBound::Unbounded) => self.compare(l) != Ordering::Less,
            (RangeBound::Excluded(l), RangeBound::Unbounded) => {
                self.compare(l) == Ordering::Greater
            }
            (RangeBound::Unbounded, RangeBound::Unbounded) => true,
        }
    }
}

// Implementation for strings
impl Collation for str {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn successor_bytes(&self) -> Option<Vec<u8>> {
        if self.is_maximum() {
            None
        } else {
            let mut bytes = self.as_bytes().to_vec();
            bytes.push(0);
            Some(bytes)
        }
    }

    fn predecessor_bytes(&self) -> Option<Vec<u8>> {
        if self.is_minimum() {
            None
        } else {
            let bytes = self.as_bytes();
            if bytes.is_empty() {
                None
            } else {
                Some(bytes[..bytes.len() - 1].to_vec())
            }
        }
    }

    fn is_minimum(&self) -> bool {
        self.is_empty()
    }

    fn is_maximum(&self) -> bool {
        false // Strings have no theoretical maximum
    }

    fn compare(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

// Implementation for integers
impl Collation for i64 {
    fn to_bytes(&self) -> Vec<u8> {
        // Use big-endian encoding to preserve ordering
        self.to_be_bytes().to_vec()
    }

    fn successor_bytes(&self) -> Option<Vec<u8>> {
        if self == &i64::MAX {
            None
        } else {
            Some((self + 1).to_be_bytes().to_vec())
        }
    }

    fn predecessor_bytes(&self) -> Option<Vec<u8>> {
        if self == &i64::MIN {
            None
        } else {
            Some((self - 1).to_be_bytes().to_vec())
        }
    }

    fn is_minimum(&self) -> bool {
        *self == i64::MIN
    }

    fn is_maximum(&self) -> bool {
        *self == i64::MAX
    }

    fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

// Implementation for floats
impl Collation for f64 {
    fn to_bytes(&self) -> Vec<u8> {
        let bits = if self.is_nan() {
            u64::MAX // NaN sorts last
        } else {
            let bits = self.to_bits();
            if *self >= 0.0 {
                bits ^ (1 << 63) // Flip sign bit for positive numbers
            } else {
                !bits // Flip all bits for negative numbers
            }
        };
        bits.to_be_bytes().to_vec()
    }

    fn successor_bytes(&self) -> Option<Vec<u8>> {
        if self.is_nan() || (self.is_infinite() && *self > 0.0) {
            None
        } else {
            let bits = if *self >= 0.0 {
                self.to_bits() ^ (1 << 63) // Apply same sign bit flip as to_bytes
            } else {
                !self.to_bits() // Apply same bit inversion as to_bytes
            };
            let next_bits = bits + 1;
            Some(next_bits.to_be_bytes().to_vec())
        }
    }

    fn predecessor_bytes(&self) -> Option<Vec<u8>> {
        if self.is_nan() || (self.is_infinite() && *self < 0.0) {
            None
        } else {
            let bits = if *self >= 0.0 {
                self.to_bits() ^ (1 << 63) // Apply same sign bit flip as to_bytes
            } else {
                !self.to_bits() // Apply same bit inversion as to_bytes
            };
            let prev_bits = bits - 1;
            Some(prev_bits.to_be_bytes().to_vec())
        }
    }

    fn is_minimum(&self) -> bool {
        *self == f64::NEG_INFINITY
    }

    fn is_maximum(&self) -> bool {
        *self == f64::INFINITY
    }

    fn compare(&self, other: &Self) -> Ordering {
        if self.is_nan() {
            if other.is_nan() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        } else if other.is_nan() {
            Ordering::Less
        } else {
            self.partial_cmp(other).unwrap_or(Ordering::Equal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_collation() {
        let s = "hello";
        assert!(s.successor_bytes().unwrap() > s.to_bytes());
        assert!(s.predecessor_bytes().unwrap() < s.to_bytes());
        assert!(!s.is_minimum());
        assert!(!s.is_maximum());

        let empty = "";
        assert!(empty.is_minimum());
        assert!(empty.predecessor_bytes().is_none());
    }

    #[test]
    fn test_integer_collation() {
        let n = 42i64;
        assert_eq!(
            i64::from_be_bytes(n.successor_bytes().unwrap().try_into().unwrap()),
            43
        );
        assert_eq!(
            i64::from_be_bytes(n.predecessor_bytes().unwrap().try_into().unwrap()),
            41
        );
        assert!(!n.is_minimum());
        assert!(!n.is_maximum());

        assert!(i64::MAX.successor_bytes().is_none());
        assert!(i64::MIN.predecessor_bytes().is_none());
        assert!(i64::MAX.is_maximum());
        assert!(i64::MIN.is_minimum());
    }

    #[test]
    fn test_float_collation() {
        let f = 1.0f64;
        assert!(f.successor_bytes().unwrap() > f.to_bytes());
        assert!(f.predecessor_bytes().unwrap() < f.to_bytes());
        assert!(!f.is_minimum());
        assert!(!f.is_maximum());

        assert!(f64::INFINITY.is_maximum());
        assert!(f64::NEG_INFINITY.is_minimum());
        assert!(f64::INFINITY.successor_bytes().is_none());
        assert!(f64::NEG_INFINITY.predecessor_bytes().is_none());

        let nan = f64::NAN;
        assert!(nan.successor_bytes().is_none());
        assert!(nan.predecessor_bytes().is_none());
    }

    #[test]
    fn test_range_bounds() {
        let n = 42i64;

        // Test inclusive bounds
        assert!(n.is_in_range(RangeBound::Included(&40), RangeBound::Included(&45)));
        assert!(n.is_in_range(RangeBound::Included(&42), RangeBound::Included(&45)));
        assert!(n.is_in_range(RangeBound::Included(&40), RangeBound::Included(&42)));

        // Test exclusive bounds
        assert!(n.is_in_range(RangeBound::Excluded(&40), RangeBound::Excluded(&43)));
        assert!(!n.is_in_range(RangeBound::Excluded(&42), RangeBound::Excluded(&43)));

        // Test mixed bounds
        assert!(n.is_in_range(RangeBound::Included(&42), RangeBound::Excluded(&43)));
        assert!(!n.is_in_range(RangeBound::Excluded(&41), RangeBound::Excluded(&42)));

        // Test unbounded
        assert!(n.is_in_range(RangeBound::Unbounded, RangeBound::Included(&45)));
        assert!(n.is_in_range(RangeBound::Included(&40), RangeBound::Unbounded));
        assert!(n.is_in_range(RangeBound::Unbounded, RangeBound::Unbounded));
    }
}
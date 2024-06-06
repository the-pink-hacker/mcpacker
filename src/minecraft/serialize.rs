use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum FloatInt {
    Integer(i64),
    Float(f32),
}

impl FloatInt {
    pub fn is_int(&self) -> bool {
        match self {
            Self::Integer(_) => true,
            Self::Float(_) => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::Integer(_) => false,
            Self::Float(_) => true,
        }
    }
}

impl From<f32> for FloatInt {
    fn from(value: f32) -> Self {
        if value.is_nan() || value > i64::MAX as f32 || value < i64::MIN as f32 {
            return Self::Float(value);
        }

        let integer = value as i64;

        if value == integer as f32 {
            Self::Integer(integer)
        } else {
            Self::Float(value)
        }
    }
}

impl From<FloatInt> for f32 {
    fn from(value: FloatInt) -> Self {
        match value {
            FloatInt::Float(float) => float,
            FloatInt::Integer(integer) => integer as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_int_whole() {
        assert_eq!(FloatInt::Integer(1), FloatInt::from(1.0));
    }

    #[test]
    fn float_int_nan() {
        let float_int = FloatInt::from(f32::NAN);
        assert!(f32::from(float_int).is_nan());
    }

    #[test]
    fn float_int_infite() {
        assert_eq!(
            FloatInt::Float(f32::INFINITY),
            FloatInt::from(f32::INFINITY)
        )
    }

    #[test]
    fn float_int_decimal() {
        assert_eq!(FloatInt::Float(1.5), FloatInt::from(1.5));
    }

    #[test]
    fn float_int_negative() {
        assert_eq!(FloatInt::Integer(-1), FloatInt::from(-1.0));
    }
}

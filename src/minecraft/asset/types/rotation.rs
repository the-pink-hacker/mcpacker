use anyhow::anyhow;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateRotation {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
}

impl<'de> Deserialize<'de> for StateRotation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StateRotationVisitor)
    }
}

impl TryFrom<u64> for StateRotation {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value % 360 {
            0 => Ok(Self::Degrees0),
            90 => Ok(Self::Degrees90),
            180 => Ok(Self::Degrees180),
            270 => Ok(Self::Degrees270),
            _ => Err(anyhow!("Not an increment of 90: {}", value)),
        }
    }
}

impl From<StateRotation> for u16 {
    fn from(value: StateRotation) -> Self {
        match value {
            StateRotation::Degrees0 => 0,
            StateRotation::Degrees90 => 90,
            StateRotation::Degrees180 => 180,
            StateRotation::Degrees270 => 270,
        }
    }
}

impl Serialize for StateRotation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.clone().into())
    }
}

struct StateRotationVisitor;

impl<'de> Visitor<'de> for StateRotationVisitor {
    type Value = StateRotation;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer at increments of 90")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        StateRotation::try_from(v).map_err(|e| E::custom(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_to_rotation_0() {
        assert_eq!(StateRotation::Degrees0, StateRotation::try_from(0).unwrap())
    }

    #[test]
    fn number_to_rotation_90() {
        assert_eq!(
            StateRotation::Degrees90,
            StateRotation::try_from(90).unwrap()
        )
    }

    #[test]
    fn number_to_rotation_180() {
        assert_eq!(
            StateRotation::Degrees180,
            StateRotation::try_from(180).unwrap()
        )
    }

    #[test]
    fn number_to_rotation_270() {
        assert_eq!(
            StateRotation::Degrees270,
            StateRotation::try_from(270).unwrap()
        )
    }

    #[test]
    fn number_to_rotation_90_alt() {
        assert_eq!(
            StateRotation::Degrees90,
            StateRotation::try_from(450).unwrap()
        )
    }

    #[test]
    fn number_to_rotation_unaligned() {
        assert!(StateRotation::try_from(1).is_err())
    }
}

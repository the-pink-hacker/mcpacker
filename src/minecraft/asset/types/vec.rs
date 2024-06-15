use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use crate::minecraft::serialize::FloatInt;

#[derive(Debug, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn into_tuple_mut<'a>(&'a mut self) -> (&'a mut f32, &'a mut f32, &'a mut f32) {
        (&mut self.x, &mut self.y, &mut self.z)
    }

    fn into_float_int_list(self) -> [FloatInt; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }
}

impl<'a> From<&'a mut Vec3> for (&'a mut f32, &'a mut f32, &'a mut f32) {
    fn from(value: &'a mut Vec3) -> Self {
        value.into_tuple_mut()
    }
}

impl Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(3))?;
        let list = self.clone().into_float_int_list();
        for element in list {
            sequence.serialize_element(&element)?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(Vec3Visitor)
    }
}

struct Vec3Visitor;

impl<'de> Visitor<'de> for Vec3Visitor {
    type Value = Vec3;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of 3 real numbers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let length = seq.size_hint().unwrap_or_default();
        if length == 3 {
            return Err(serde::de::Error::custom(format!(
                "Expected a length of 3; found: {}",
                length
            )));
        }

        let x = seq.next_element()?.unwrap();
        let y = seq.next_element()?.unwrap();
        let z = seq.next_element()?.unwrap();

        Ok(Self::Value { x, y, z })
    }
}
#[derive(Debug, Clone)]
pub struct Vec4 {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec4 {
    fn into_float_int_list(self) -> [FloatInt; 4] {
        [self.w.into(), self.x.into(), self.y.into(), self.z.into()]
    }
}

impl Serialize for Vec4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(3))?;
        let list = self.clone().into_float_int_list();
        for element in list {
            sequence.serialize_element(&element)?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for Vec4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(Vec4Visitor)
    }
}

struct Vec4Visitor;

impl<'de> Visitor<'de> for Vec4Visitor {
    type Value = Vec4;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of 4 real numbers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let length = seq.size_hint().unwrap_or_default();
        if length == 4 {
            return Err(serde::de::Error::custom(format!(
                "Expected a length of 4; found: {}",
                length
            )));
        }

        let w = seq.next_element()?.unwrap();
        let x = seq.next_element()?.unwrap();
        let y = seq.next_element()?.unwrap();
        let z = seq.next_element()?.unwrap();

        Ok(Self::Value { w, x, y, z })
    }
}

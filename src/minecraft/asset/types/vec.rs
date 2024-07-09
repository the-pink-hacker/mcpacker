use std::ops::{Add, AddAssign};

use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};

use crate::minecraft::serialize::FloatInt;

#[derive(Debug, PartialEq, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl<'a> From<&'a mut Vec3> for (&'a mut f32, &'a mut f32, &'a mut f32) {
    fn from(value: &'a mut Vec3) -> Self {
        (&mut value.x, &mut value.y, &mut value.z)
    }
}

impl<'a> From<&'a Vec3> for (&'a f32, &'a f32, &'a f32) {
    fn from(value: &'a Vec3) -> Self {
        (&value.x, &value.y, &value.z)
    }
}

impl From<Vec3> for (f32, f32, f32) {
    fn from(value: Vec3) -> Self {
        (value.x, value.y, value.z)
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(value: (f32, f32, f32)) -> Self {
        let (x, y, z) = value;
        Self { x, y, z }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self
    }
}

impl Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&FloatInt::from(self.x))?;
        tuple.serialize_element(&FloatInt::from(self.y))?;
        tuple.serialize_element(&FloatInt::from(self.z))?;
        tuple.end()
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
        let x = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let y = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let z = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

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

impl<'a> From<&'a mut Vec4> for (&'a mut f32, &'a mut f32, &'a mut f32, &'a mut f32) {
    fn from(value: &'a mut Vec4) -> Self {
        (&mut value.w, &mut value.x, &mut value.y, &mut value.z)
    }
}

impl<'a> From<&'a Vec4> for (&'a f32, &'a f32, &'a f32, &'a f32) {
    fn from(value: &'a Vec4) -> Self {
        (&value.w, &value.x, &value.y, &value.z)
    }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(value: Vec4) -> Self {
        (value.w, value.x, value.y, value.z)
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        let (w, x, y, z) = value;
        Self { w, x, y, z }
    }
}

impl Serialize for Vec4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&FloatInt::from(self.w))?;
        tuple.serialize_element(&FloatInt::from(self.x))?;
        tuple.serialize_element(&FloatInt::from(self.y))?;
        tuple.serialize_element(&FloatInt::from(self.z))?;
        tuple.end()
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
        let w = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let x = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let y = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
        let z = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;

        Ok(Self::Value { w, x, y, z })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3_add() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(2.0, 3.0, 1.0);
        let expect = Vec3::new(3.0, 5.0, 4.0);
        assert_eq!(a + b, expect);
    }

    #[test]
    fn vec3_add_assign() {
        let mut a = Vec3::new(1.0, 2.0, 3.0);
        a += Vec3::new(2.0, 3.0, 1.0);
        let expect = Vec3::new(3.0, 5.0, 4.0);
        assert_eq!(a, expect);
    }
}

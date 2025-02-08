use chrono::{DateTime, Utc};
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;

pub trait JsonParseable<T: PartialEq + PartialOrd> {
    fn parse(json: &JsonValue) -> Option<T>;
}

impl JsonParseable<u8> for u8 {
    fn parse(json: &JsonValue) -> Option<u8> {
        json.as_u8()
    }
}

impl JsonParseable<u16> for u16 {
    fn parse(json: &JsonValue) -> Option<u16> {
        json.as_u16()
    }
}

impl JsonParseable<u32> for u32 {
    fn parse(json: &JsonValue) -> Option<u32> {
        json.as_u32()
    }
}

impl JsonParseable<u64> for u64 {
    fn parse(json: &JsonValue) -> Option<u64> {
        json.as_u64()
    }
}

impl JsonParseable<u128> for u128 {
    fn parse(json: &JsonValue) -> Option<u128> {
        json.as_u64().map(|x| x as u128)
    }
}

// Signed integers
impl JsonParseable<i8> for i8 {
    fn parse(json: &JsonValue) -> Option<i8> {
        json.as_i64().and_then(|x| i8::try_from(x).ok())
    }
}

impl JsonParseable<i16> for i16 {
    fn parse(json: &JsonValue) -> Option<i16> {
        json.as_i64().and_then(|x| i16::try_from(x).ok())
    }
}

impl JsonParseable<i32> for i32 {
    fn parse(json: &JsonValue) -> Option<i32> {
        json.as_i32()
    }
}

impl JsonParseable<i64> for i64 {
    fn parse(json: &JsonValue) -> Option<i64> {
        json.as_i64()
    }
}

impl JsonParseable<i128> for i128 {
    fn parse(json: &JsonValue) -> Option<i128> {
        json.as_i64().map(|x| x as i128)
    }
}

// Floating point
impl JsonParseable<f32> for f32 {
    fn parse(json: &JsonValue) -> Option<f32> {
        json.as_f64().map(|x| x as f32)
    }
}

impl JsonParseable<f64> for f64 {
    fn parse(json: &JsonValue) -> Option<f64> {
        json.as_f64()
    }
}

// Boolean
impl JsonParseable<bool> for bool {
    fn parse(json: &JsonValue) -> Option<bool> {
        json.as_bool()
    }
}

// String
impl JsonParseable<String> for String {
    fn parse(json: &JsonValue) -> Option<String> {
        json.as_str().map(String::from)
    }
}

// DateTime
impl JsonParseable<DateTime<Utc>> for DateTime<Utc> {
    fn parse(json: &JsonValue) -> Option<DateTime<Utc>> {
        json.as_str().map(|s| {
            DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
                .map(|x| x.with_timezone(&Utc))
                .ok()
        })
        .flatten()
    }
}

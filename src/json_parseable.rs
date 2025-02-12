use chrono::{DateTime, FixedOffset};
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;

pub trait JsonParseable<T: PartialEq + PartialOrd> {
    fn parse(json: &JsonValue) -> Result<T, String>;
}

macro_rules! generate_json_parseable {   
    ($prefix:ident, $size:expr) => {
        paste::paste! {            
            impl JsonParseable<[<$prefix $size>]> for [<$prefix $size>] {
                #[inline (always)]
                fn parse(json: &JsonValue) -> Result<[<$prefix $size>], String> {
                    [<$prefix $size>]::try_from([<parse_ $prefix 64>](json)?).map_err(|e| e.to_string())
                }
            }
        }
    };
}

macro_rules! generate_json_parser {
    ($prefix:ident) => {
        paste::paste! {
            #[inline (always)]
            fn [<parse_ $prefix 64>](json: &JsonValue) -> Result<[<$prefix 64>], String> {
                json.[<as_ $prefix 64>]().ok_or(format!("Failed to get JSON as {}", stringify!([<$prefix 64>])))
            }
        }
    };
}

generate_json_parser!(i);
generate_json_parser!(u);
generate_json_parser!(f);

generate_json_parseable!(i, 8);
generate_json_parseable!(i, 16);
generate_json_parseable!(i, 32);
generate_json_parseable!(i, 64);

generate_json_parseable!(u, 8);
generate_json_parseable!(u, 16);
generate_json_parseable!(u, 32);
generate_json_parseable!(u, 64);


// Floating point
impl JsonParseable<f32> for f32 {
    #[inline (always)]
    fn parse(json: &JsonValue) -> Result<f32, String> {
        parse_f64(json).map(|x| x as f32)
    }
}

impl JsonParseable<f64> for f64 {
    #[inline (always)]
    fn parse(json: &JsonValue) -> Result<f64, String> {
        parse_f64(json)
    }
}



impl JsonParseable<bool> for bool {
    fn parse(json: &JsonValue) -> Result<bool, String> {
        json.as_bool().ok_or("Failed to get JSON as bool".to_string())
    }
}

impl JsonParseable<String> for String {
    fn parse(json: &JsonValue) -> Result<String, String> {
        json.as_str().map(String::from).ok_or("Failed to get JSON as String".to_string())
    }
}
impl JsonParseable<DateTime<FixedOffset>> for DateTime<FixedOffset> {
    fn parse(json: &JsonValue) -> Result<DateTime<FixedOffset>, String> {
        let res_str = json.as_str()
            .ok_or("Failed to get JSON as String".to_string())?;
        DateTime::parse_from_str(res_str, "%d.%m.%YT%H:%M:%S%:z")
            .map_err(|e| e.to_string())
    }

}


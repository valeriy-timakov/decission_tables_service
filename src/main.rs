extern crate core;


use std::borrow::Cow;
use std::collections::HashMap;
use std::env::var;
use std::fmt::format;
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;
use decision_tables_service::json_parseable::JsonParseable;
use decision_tables_service::conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};
use decision_tables_service::decession_table::{DecisionTable, DecisionTableSource, Rule, RuleTrait};
use calamine::{Reader, open_workbook, Xlsx, DataType, Range, Sheet};
use std::path::Path;
use std::f64;
use std::iter::Map;
use chrono::{DateTime, Utc};
use decision_tables_service::xlsx_dt_data_source::{ParsePreferences, XlsxDTDataSource};

fn main() {

    let mut data = br#"{
        "parameters": {
            "dateFrom": "06.11.2024",  
            "value": 12,    
            "label": ""
        }
    }"#.to_vec();

    // Парсимо JSON
    let mut parsed: JsonValue = simd_json::from_slice(&mut data).expect("Failed to parse JSON");
        
    let parse_preferences = ParsePreferences::new(
        ',', DataType::String("*".to_string()), "%Y-%m-%dT%H:%M:%S".to_string());
    let mut loader = XlsxDTDataSource::new(
        "C:\\Users\\valti\\Downloads\\test_dt.xlsx".to_string(), parse_preferences);
    loader.init().unwrap();
    
    let dt = DecisionTable::create(Box::new(loader)).unwrap();
    
    dt.check_all(&parsed).iter().for_each(|x| println!("Res: {}", x));
}

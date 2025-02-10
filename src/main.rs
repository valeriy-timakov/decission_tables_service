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
            "name": "someName",  
            "age": 30      
        }
    }"#.to_vec();

    // Парсимо JSON
    let mut parsed: JsonValue = simd_json::from_slice(&mut data).expect("Failed to parse JSON");

    // Дістаємо значення "age"
    if let Some(age) = parsed["parameters"]["age"].as_u8() {
        println!("Age: {}", age);
        
        let parse_preferences = ParsePreferences::new(
            ',', DataType::String("*".to_string()), "%Y-%m-%dT%H:%M:%S".to_string());
        let mut loader = XlsxDTDataSource::new(
            "C:\\Users\\valti\\Downloads".to_string(), parse_preferences);
        loader.init().unwrap();
        
        let dt = DecisionTable::create(Box::new(loader)).unwrap();

        let equal_rule = 
            Rule::new("age".to_string(), Box::new(EqualCondition {}), vec!(23_u8, 25, 30, 35, 40).iter().map(|x| Some(*x)).collect());

        let more_than_rule = 
            Rule::new("age".to_string(), Box::new(GreaterThanCondition {}), vec!(18_u8, 20, 22, 25, 30, 45, 90).iter().map(|x| Some(*x)).collect());

        let greater_than_or_equal_rule = 
            Rule::new("age".to_string(), Box::new(GreaterThanOrEqualCondition {}), vec!(18_u8, 25, 30).iter().map(|x| Some(*x)).collect());

        let less_than_rule = 
            Rule::new("age".to_string(), Box::new(LessThanCondition {}), vec!(40_u8, 50, 60).iter().map(|x| Some(*x)).collect());

        let less_than_or_equal_rule = 
            Rule::new("age".to_string(), Box::new(LessThanOrEqualCondition {}), vec!(30_u8, 35, 40).iter().map(|x| Some(*x)).collect());

        let item = &parsed["parameters"];
        match equal_rule.check_all(item) {
            Some(indices) => println!("Equal rule matched at indices: {:?}", indices),
            None => println!("Error parsing value: {}", item),
        }

        match greater_than_or_equal_rule.check_all(item) {
            Some(indices) => println!("Greater than or equal rule matched at indices: {:?}", indices),
            None => println!("Error parsing value: {}", item),
        }

        match less_than_rule.check_all(item) {
            Some(indices) => println!("Less than rule matched at indices: {:?}", indices),
            None => println!("Error parsing value: {}", item),
        }

        match less_than_or_equal_rule.check_all(item) {
            Some(indices) => println!("Less than or equal rule matched at indices: {:?}", indices),
            None => println!("Error parsing value: {}", item),
        }

        match more_than_rule.check_all(item) {
            Some(indices) => println!("More than rule matched at indices: {:?}", indices),
            None => println!("Error parsing value: {}", item),
        }
    } else {
        println!("Field 'age' not found or not an integer");
    }
}

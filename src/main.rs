extern crate core;

mod json_parseable;
mod conditions;

use std::env::var;
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;
use json_parseable::JsonParseable;
use conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};

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

trait RuleTrait {
    fn check_all(&self, value: &JsonValue) -> Option<Vec<usize>>;
}

struct Rule<T: PartialEq + PartialOrd> {
    fieldName: String,
    condition: Box<dyn Condition<T>>,
    values: Vec<Option<T>>,
}

impl<T: PartialEq + PartialOrd> Rule<T> {
    fn new(fieldName: String, condition: Box<dyn Condition<T>>, values: Vec<Option<T>>) -> Rule<T> {
        Rule { fieldName, condition, values }
    }
}

impl<T: PartialEq + PartialOrd + JsonParseable<T>> RuleTrait for Rule<T> {
    fn check_all(&self, value: &JsonValue) -> Option<Vec<usize>> {
        let t_value: Option<T> = T::parse(&value[self.fieldName.as_str()]);
        t_value.map(|t_value| {
            let mut result: Vec<usize> = Vec::new();
            for (index, val_opt) in self.values.iter().enumerate() {
                if 
                    val_opt.as_ref()
                        .map(|val| {self.condition.check(&t_value, val)})
                        .unwrap_or(true) 
                {
                    result.push(index);
                }
            }
            result
        }) 
    }
}

struct StringData {
    max_length: usize,
    contains_utf: bool,
}

struct NumberData {
    has_negative: bool,
    max_value: i128,
    has_decimal: bool, 
    has_f64: bool,
}

enum ValueCondition {
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
}

enum ValueType {
    String(StringData),
    Number(NumberData),
    Boolean,
}

struct RuleData {
    field_name: String,
    condition: ValueCondition,
    field_type: ValueType,
}

trait DecisionTableSource {
    fn init(&mut self) -> Result<(), String>;
    fn get_rules_count(&self) -> usize;
    fn get_variants_count(&self) -> usize;
    fn get_rule_data(&self, rule_num: usize) -> &RuleData;
    fn get_result_datas(&self) -> Vec<String>;
    fn get_u8_rule_values(&self, rule_num: usize) -> Vec<Option<u8>>;
    fn get_u16_rule_values(&self, rule_num: usize) -> Vec<Option<u16>>;
    fn get_u32_rule_values(&self, rule_num: usize) -> Vec<Option<u32>>;
    fn get_u64_rule_values(&self, rule_num: usize) -> Vec<Option<u64>>;
    fn get_i8_rule_values(&self, rule_num: usize) -> Vec<Option<i8>>;
    fn get_i16_rule_values(&self, rule_num: usize) -> Vec<Option<i16>>;
    fn get_i32_rule_values(&self, rule_num: usize) -> Vec<Option<i32>>;
    fn get_i64_rule_values(&self, rule_num: usize) -> Vec<Option<i64>>;
    fn get_f32_rule_values(&self, rule_num: usize) -> Vec<Option<f32>>;
    fn get_f64_rule_values(&self, rule_num: usize) -> Vec<Option<f64>>;
    fn get_string_rule_values(&self, rule_num: usize) -> Vec<Option<String>>;
    fn get_bool_rule_values(&self, rule_num: usize) -> Vec<Option<bool>>;
    // fn iterate_rules<F: FnMut(&dyn it)>(&self, callback: F);
    
}

struct DecisionTable {
    rules: Vec<Box<dyn RuleTrait>>,
    data: Vec<String>, 
}

fn map_conditions<T: PartialEq + PartialOrd>(condition: &ValueCondition) -> Box<dyn Condition<T>> {
    match condition { 
        ValueCondition::LessThan => Box::new(LessThanCondition {}),
        ValueCondition::LessOrEqual => Box::new(LessThanOrEqualCondition {}),
        ValueCondition::GreaterThan => Box::new(GreaterThanCondition {}),
        ValueCondition::GreaterOrEqual => Box::new(GreaterThanOrEqualCondition {}),
        ValueCondition::Equal => Box::new(EqualCondition {})
    }
}

fn create_rule<T: PartialEq + PartialOrd>(rule_data: &RuleData, values: Vec<Option<T>>) -> Rule<T> {
    Rule::new(rule_data.field_name.clone(), map_conditions(&rule_data.condition), values)
}

fn create_rule_for_type(data_source: &Box<dyn DecisionTableSource>, rule_num: usize) -> Box<dyn RuleTrait> {
    let rule_data: &RuleData = data_source.get_rule_data(rule_num);
    match &rule_data.field_type {  
        ValueType::String(type_data) => Box::new(create_rule(rule_data, data_source.get_string_rule_values(rule_num))),
        ValueType::Boolean => Box::new(create_rule(rule_data, data_source.get_bool_rule_values(rule_num))),
        ValueType::Number(type_data) => {
            if type_data.has_decimal {
                if type_data.has_f64 {
                    Box::new(create_rule(rule_data, data_source.get_f64_rule_values(rule_num)))
                } else {
                    Box::new(create_rule(rule_data, data_source.get_f32_rule_values(rule_num)))
                }
            } else {
                if type_data.has_negative {
                    if type_data.max_value <= i8::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_i8_rule_values(rule_num)))
                    } else if type_data.max_value <= i16::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_i16_rule_values(rule_num)))
                    } else if type_data.max_value <= i32::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_i32_rule_values(rule_num)))
                    } else {
                        Box::new(create_rule(rule_data, data_source.get_i64_rule_values(rule_num)))
                    }
                } else {
                    if type_data.max_value <= u8::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_u8_rule_values(rule_num)))
                    } else if type_data.max_value <= u16::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_u16_rule_values(rule_num)))
                    } else if type_data.max_value <= u32::MAX as i128 {
                        Box::new(create_rule(rule_data, data_source.get_u32_rule_values(rule_num)))
                    } else {
                        Box::new(create_rule(rule_data, data_source.get_u64_rule_values(rule_num)))
                    }
                }
            }
        }
    }
}

impl DecisionTable {
    fn create(mut data_source: Box<dyn DecisionTableSource>) -> Result<DecisionTable, String> {
        let vaiants_count = data_source.get_rules_count();
        let rules_count = data_source.get_rules_count();
        let mut rules: Vec<Box<dyn RuleTrait>> = Vec::new();
        for i in 0..rules_count {
            rules.push( create_rule_for_type(&data_source, i) );
        }
        let data = data_source.get_result_datas();
        if vaiants_count != data.len() {
            return Err(format!("Error in DecisionTable data! Found {} variants count, where required {}", 
                               data.len(), vaiants_count));
        }
        Ok(DecisionTable {
            rules,
            data,
        })            
    }
}

struct XlsxDTDataSource {}


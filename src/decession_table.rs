use crate::conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};
use crate::json_parseable::JsonParseable;
use chrono::{DateTime, FixedOffset};
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueObjectAccess;
use std::f64;

pub trait RuleTrait {
    fn check_all(&self, value: &JsonValue) -> Result<Vec<usize>, String>;
    fn check_indices(&self, value: &JsonValue, indices: &Vec<usize>) -> Result<Vec<usize>, String>;
}

pub struct Rule<T: PartialEq + PartialOrd> {
    field_name: String,
    condition: Box<dyn Condition<T>>,
    values: Vec<Option<T>>,
}

impl<T: PartialEq + PartialOrd> Rule<T> {
    pub fn new(field_name: String, condition: Box<dyn Condition<T>>, values: Vec<Option<T>>) -> Rule<T> {
        Rule { field_name, condition, values }
    }
}

impl<T: PartialEq + PartialOrd + JsonParseable<T>> RuleTrait for Rule<T> {
    fn check_all(&self, value: &JsonValue) -> Result<Vec<usize>, String> {
        let t_value: T = value.get(self.field_name.as_str())
            .ok_or(format!("Error in query data! Field '{}' not found: {}", self.field_name, value))
            
            .and_then(|x| T::parse(x)
                .map_err(|e| format!("Error converting query field {} to {} rule type! {}", x, self.field_name, e))
            )?;
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
        Ok(result)
    }


    fn check_indices(&self, value: &JsonValue, indices: &Vec<usize>) -> Result<Vec<usize>, String> {
        let t_value: T = value.get(self.field_name.as_str())
            .ok_or(format!("Error in query data! Field '{}' not found: {}", self.field_name, value))

            .and_then(|x| T::parse(x)
                .map_err(|e| format!("Error converting query field {} to {} rule type! {}", x, self.field_name, e))
            )?;
        let mut result: Vec<usize> = Vec::new();
        let iterator = indices.iter().map(|i|
            (*i, self.values.get(*i).expect(format!("Error in DecisionTable data! Index out of bounds: {}", *i).as_str())));
        for (index, val_opt) in iterator {
            if
                val_opt.as_ref()
                    .map(|val| {self.condition.check(&t_value, val)})
                    .unwrap_or(true)
            {
                result.push(index);
            }
        }
        Ok(result)
    }
}

pub enum DecimalType {
    F32,
    F64,
}

pub struct IntegerData {
    has_negative: bool,
    max_value: i128
}

impl IntegerData {
    pub fn new(has_negative: bool, max_value: i128) -> IntegerData {
        IntegerData { has_negative, max_value }
    }
}

pub struct StringData {
    max_length: usize,
    contains_utf: bool,
}

impl StringData {
    pub fn new(max_length: usize, contains_utf: bool) -> StringData {
        StringData { max_length, contains_utf }
    }
}

pub enum NumberType {
    Integer(IntegerData),
    Decimal(DecimalType),
}

pub enum ValueCondition {
    LessThan,
    GreaterThan,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
}

pub enum ValueType {
    String(StringData),
    Number(NumberType),
    DateTime(FixedOffset),
    Boolean,
}

pub struct RuleData {
    field_name: String,
    condition: ValueCondition,
    pub field_type: ValueType,
}

impl RuleData {
    pub fn new(field_name: String, condition: ValueCondition, field_type: ValueType) -> RuleData {
        RuleData { field_name, condition, field_type }
    }
}

pub trait DecisionTableSource {
    fn init(&mut self) -> Result<(), String>;
    fn get_rules_count(&self) -> Result<usize, String>;
    fn get_variants_count(&self) -> Result<usize, String>;
    fn get_rule_data(&self, rule_num: usize) -> Result<&RuleData, String>;
    fn get_result_datas(&self) -> Result<Vec<String>, String>;
    fn get_u8_rule_values(&self, rule_num: usize) -> Result<Vec<Option<u8>>, String>;
    fn get_u16_rule_values(&self, rule_num: usize) -> Result<Vec<Option<u16>>, String>;
    fn get_u32_rule_values(&self, rule_num: usize) -> Result<Vec<Option<u32>>, String>;
    fn get_u64_rule_values(&self, rule_num: usize) -> Result<Vec<Option<u64>>, String>;
    fn get_i8_rule_values(&self, rule_num: usize) -> Result<Vec<Option<i8>>, String>;
    fn get_i16_rule_values(&self, rule_num: usize) -> Result<Vec<Option<i16>>, String>;
    fn get_i32_rule_values(&self, rule_num: usize) -> Result<Vec<Option<i32>>, String>;
    fn get_i64_rule_values(&self, rule_num: usize) -> Result<Vec<Option<i64>>, String>;
    fn get_f32_rule_values(&self, rule_num: usize) -> Result<Vec<Option<f32>>, String>;
    fn get_f64_rule_values(&self, rule_num: usize) -> Result<Vec<Option<f64>>, String>;
    fn get_string_rule_values(&self, rule_num: usize) -> Result<Vec<Option<String>>, String>;
    fn get_bool_rule_values(&self, rule_num: usize) -> Result<Vec<Option<bool>>, String>;
    fn get_date_time_rule_values(&self, rule_num: usize) -> Result<Vec<Option<DateTime<FixedOffset>>>, String>;
    // fn iterate_rules<F: FnMut(&dyn it)>(&self, callback: F);

}

pub struct DecisionTable {
    rules: Vec<Box<dyn RuleTrait>>,
    data: Vec<String>,
    variants_count: usize,
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

fn create_rule<T: PartialEq + PartialOrd>(rule_data: &RuleData, values: Vec<Option<T>>, variants_count: usize) -> Result<Rule<T>, String> {
    if values.len() != variants_count {
        return Err(format!("Error in DecisionTable data, rule {}! Found {} values, where required {}!", 
           rule_data.field_name, values.len(), variants_count));
    }
    Ok(Rule::new(rule_data.field_name.clone(), map_conditions(&rule_data.condition), values))
}

fn create_rule_for_type(data_source: &Box<dyn DecisionTableSource>, rule_num: usize, variants_count: usize) -> Result<Box<dyn RuleTrait>, String> {
    let rule_data: &RuleData = data_source.get_rule_data(rule_num)?;
    match &rule_data.field_type {
        ValueType::String(_) => Ok(Box::new(create_rule(rule_data, data_source.get_string_rule_values(rule_num)?, variants_count)?)),
        ValueType::Boolean => Ok(Box::new(create_rule(rule_data, data_source.get_bool_rule_values(rule_num)?, variants_count)?)),
        ValueType::DateTime(_) => Ok(Box::new(create_rule(rule_data, data_source.get_date_time_rule_values(rule_num)?, variants_count)?)),
        ValueType::Number(number_type) => {
            match number_type {
                NumberType::Decimal(decimal_type) => {
                    match decimal_type {
                        DecimalType::F64 => Ok(Box::new(create_rule(rule_data, data_source.get_f64_rule_values(rule_num)?, variants_count)?)),
                        DecimalType::F32 => Ok(Box::new(create_rule(rule_data, data_source.get_f32_rule_values(rule_num)?, variants_count)?)),
                    }
                },
                NumberType::Integer(integer_data) => {
                    if integer_data.has_negative {
                        if integer_data.max_value <= i8::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i8_rule_values(rule_num)?, variants_count)?))
                        } else if integer_data.max_value <= i16::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i16_rule_values(rule_num)?, variants_count)?))
                        } else if integer_data.max_value <= i32::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i32_rule_values(rule_num)?, variants_count)?))
                        } else {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i64_rule_values(rule_num)?, variants_count)?))
                        }
                    } else {
                        if integer_data.max_value <= u8::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u8_rule_values(rule_num)?, variants_count)?))
                        } else if integer_data.max_value <= u16::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u16_rule_values(rule_num)?, variants_count)?))
                        } else if integer_data.max_value <= u32::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u32_rule_values(rule_num)?, variants_count)?))
                        } else {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u64_rule_values(rule_num)?, variants_count)?))
                        }
                    }
                }
            }
        }
    }
}

impl DecisionTable {
    pub fn create(mut data_source: Box<dyn DecisionTableSource>) -> Result<DecisionTable, String> {
        let variants_count = data_source.get_variants_count()?;
        let rules_count = data_source.get_rules_count()?;
        let mut rules: Vec<Box<dyn RuleTrait>> = Vec::new();
        for i in 0..rules_count {
            rules.push( create_rule_for_type(&data_source, i, variants_count)? );
        }
        let data = data_source.get_result_datas()?;
        if variants_count != data.len() {
            return Err(format!("Error in DecisionTable data! Found {} variants count, where required {}",
                               data.len(), variants_count));
        }
        Ok(DecisionTable {
            rules,
            data,
            variants_count, 
        })
    }
    
    pub fn check_all(&self, value: &JsonValue) -> Result<Vec<String>, String> {
        let mut result: Vec<String> = Vec::new();
        let mut indices: Vec<usize> = (0..self.variants_count).collect();
        for rule in &self.rules {
            match rule.check_indices(value, &indices) {
                Ok(indices_new) => {
                    indices = indices_new;
                },
                Err(e) => return Err(e)
            }
        } 
        for i in indices {
            result.push(self.data[i].clone());
        }
        Ok(result)
    }
}

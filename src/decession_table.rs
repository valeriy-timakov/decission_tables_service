use std::borrow::Cow;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use crate::conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};
use crate::json_parseable::JsonParseable;
use chrono::{DateTime, FixedOffset};
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueObjectAccess;
use std::f64;
use std::hash::Hash;
use std::iter::Map;
use std::sync::Arc;
use crossbeam::thread;

pub trait RuleTrait: Send + Sync {
    fn check_all(&self, value: &JsonValue) -> Result<Vec<usize>, String>;
    fn check_all_multi_thread(&self, value: &JsonValue, indices: &Vec<usize>, threads_count: u8) -> Result<Vec<usize>, String>;
    fn check_indices(&self, value: &JsonValue, indices: &Vec<usize>) -> Result<Vec<usize>, String>;
}

pub trait CachedRuleTrait: RuleTrait {
    fn check_all_cached(&mut self, value: &JsonValue) -> Result<&Vec<usize>, String>;
}

pub struct Rule<T: PartialEq + PartialOrd> {
    field_name: String,
    condition: Box<dyn Condition<T>>,
    values: Vec<Option<T>>,
    cache: HashMap<T, Vec<usize>>,
}

impl <T: PartialEq + PartialOrd + JsonParseable<T>> Rule<T> {

    pub fn new(field_name: String, condition: Box<dyn Condition<T>>, values: Vec<Option<T>>) -> Rule<T> {
        Rule {
            field_name,
            condition,
            values,
            cache: HashMap::new(),
        }
    }
    pub fn parse_in_value(&self, value: &JsonValue) -> Result<T, String> {
        value.get(self.field_name.as_str())
            .ok_or(format!("Error in query data! Field '{}' not found: {}", self.field_name, value))

            .and_then(|x| T::parse(x)
                .map_err(|e| format!("Error converting query field {} to {} rule type! {}", x, self.field_name, e))
            )
    }

    fn check_slice(&self, values: &[Option<T>], t_value: &T, start_at: usize) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        for (index, val_opt) in values.iter().enumerate() {
            if val_opt.as_ref()
                .map(|val| {self.condition.check(t_value, val)})
                .unwrap_or(true)
            {
                result.push(start_at + index);
            }
        }
        result
    }

    fn check_slice_destructed(condition: &Box<dyn Condition<T>>, values: &[Option<T>], t_value: &T, start_at: usize) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        for (index, val_opt) in values.iter().enumerate() {
            if val_opt.as_ref()
                .map(|val| {condition.check(t_value, val)})
                .unwrap_or(true)
            {
                result.push(start_at + index);
            }
        }
        result
    }


    fn check_indices_slice(&self, t_value: &T, indices: &[usize]) -> Vec<usize> {
        let mut result: Vec<usize> = Vec::new();
        let iterator = indices.iter().map(|i|
            (*i, self.values.get(*i).expect(format!("Error in DecisionTable data! Index out of bounds: {}", *i).as_str())));
        for (index, val_opt) in iterator {
            if
            val_opt.as_ref()
                .map(|val| {self.condition.check(t_value, val)})
                .unwrap_or(true)
            {
                result.push(index);
            }
        }
        result
    }
}

impl<T: PartialEq + PartialOrd + JsonParseable<T> + Send + Sync + Eq + Hash + Clone> CachedRuleTrait for Rule<T> {

    fn check_all_cached(&mut self, value: &JsonValue) -> Result<&Vec<usize>, String>  {
        let t_value: T = self.parse_in_value(value)?;
        
        let Rule { ref mut cache, values, condition, .. } = self;        
        let resulst = cache.entry(t_value.clone()).or_insert_with(|| Self::check_slice_destructed(condition, &values[..], &t_value, 0));

        Ok(resulst)
    }
    
}

impl<T: PartialEq + PartialOrd + JsonParseable<T> + Send + Sync> RuleTrait for Rule<T> {
    fn check_all(&self, value: &JsonValue) -> Result<Vec<usize>, String> {
        let t_value: T = self.parse_in_value(value)?;
        let result: Vec<usize> = self.check_slice(&self.values, &t_value, 0);
        Ok(result)
    }

    fn check_all_multi_thread(&self, value: &JsonValue, indices: &Vec<usize>, threads_count: u8) -> Result<Vec<usize>, String> {
        let t_value: T = self.parse_in_value(value)?;
        thread::scope(|s| {
            let mut handles = vec![];
            let chunk_size = indices.len() / threads_count as usize;

            if chunk_size == 0 {
                return  vec![];
            }

            for chunk in indices.chunks(chunk_size) {
                let t_value_ref = &t_value;
                handles.push(s.spawn(move |_| self.check_indices_slice(t_value_ref, chunk)));
            }

            handles.into_iter().flat_map(|h| h.join().unwrap()).collect::<Vec<_>>()
        }).map_err(|e| "Thread error".to_string())
    }


    fn check_indices(&self, value: &JsonValue, indices: &Vec<usize>) -> Result<Vec<usize>, String> {
        let t_value: T = self.parse_in_value(value)?;
        let result: Vec<usize> = self.check_indices_slice(&t_value, indices);
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
    fn get_result_datas(&self) -> Result<Box<dyn OrderedSeq<String>>, String>;
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

pub trait OrderedSeq<T: Clone>: Send + Sync {
    fn item(&self, index: usize) -> Option<Cow<T>>;
    
    fn len(&self) -> usize;
}

impl <T: Clone + Send + Sync> OrderedSeq<T> for Vec<T> {

    #[inline(always)]
    fn item(&self, index: usize) -> Option<Cow<T>> {
        self.get(index).map(|x| Cow::Borrowed(x))
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
}

pub struct CSVData {
    names: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CSVData {
    pub(crate) fn new(names: Vec<String>, data: Vec<Vec<String>>) -> CSVData {
        CSVData {
            names, 
            data, 
        }
    }
}

impl OrderedSeq<String> for CSVData {
    fn item(&self, index: usize) -> Option<Cow<String>> {
        self.data.get(index).and_then(|x| {
            let mut result = String::new();
            result.push_str("{");
            for (i, (name, value)) in self.names.iter().zip(x.iter()).enumerate() {
                result.push_str("\"");
                result.push_str(name);
                result.push_str("\": \"");
                result.push_str(value);
                result.push_str("\"");
                if i + 1 < self.names.len() {
                    result.push_str(", ");
                }
            }
            result.push_str("}");
            Some(Cow::Owned(result))
        })
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct DecisionTable {
    rules: Vec<Box<dyn RuleTrait>>,
    cached_rules: Vec<Box<dyn CachedRuleTrait>>, 
    data: Box<dyn OrderedSeq<String>>,
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

fn create_rule<T: PartialEq + PartialOrd + JsonParseable<T>>(rule_data: &RuleData, values: Vec<Option<T>>, variants_count: usize) -> Result<Rule<T>, String> {
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

fn create_cached_rule_for_type(data_source: &Box<dyn DecisionTableSource>, rule_num: usize, variants_count: usize) -> Result<Option<Box<dyn CachedRuleTrait>>, String> {
    let rule_data: &RuleData = data_source.get_rule_data(rule_num)?;
    match &rule_data.field_type {
        ValueType::String(_) => Ok(Some(Box::new(create_rule(rule_data, data_source.get_string_rule_values(rule_num)?, variants_count)?))),
        ValueType::Boolean => Ok(Some(Box::new(create_rule(rule_data, data_source.get_bool_rule_values(rule_num)?, variants_count)?))),
        ValueType::DateTime(_) => Ok(Some(Box::new(create_rule(rule_data, data_source.get_date_time_rule_values(rule_num)?, variants_count)?))),
        ValueType::Number(number_type) => {
            match number_type {
                NumberType::Decimal(_) => {
                    Ok(None)
                },
                NumberType::Integer(integer_data) => {
                    if integer_data.has_negative {
                        if integer_data.max_value <= i8::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_i8_rule_values(rule_num)?, variants_count)?)))
                        } else if integer_data.max_value <= i16::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_i16_rule_values(rule_num)?, variants_count)?)))
                        } else if integer_data.max_value <= i32::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_i32_rule_values(rule_num)?, variants_count)?)))
                        } else {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_i64_rule_values(rule_num)?, variants_count)?)))
                        }
                    } else {
                        if integer_data.max_value <= u8::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_u8_rule_values(rule_num)?, variants_count)?)))
                        } else if integer_data.max_value <= u16::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_u16_rule_values(rule_num)?, variants_count)?)))
                        } else if integer_data.max_value <= u32::MAX as i128 {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_u32_rule_values(rule_num)?, variants_count)?)))
                        } else {
                            Ok(Some(Box::new(create_rule(rule_data, data_source.get_u64_rule_values(rule_num)?, variants_count)?)))
                        }
                    }
                }
            }
        }
    }
}



impl <'a> DecisionTable {
    pub fn create(mut data_source: Box<dyn DecisionTableSource>) -> Result<DecisionTable, String> {
        let variants_count = data_source.get_variants_count()?;
        let rules_count = data_source.get_rules_count()?;
        let mut rules: Vec<Box<dyn RuleTrait>> = Vec::new();
        let mut cached_rules: Vec<Box<dyn CachedRuleTrait>> = Vec::new();
        for i in 0..rules_count {
            if let Some(rule) = create_cached_rule_for_type(&data_source, i, variants_count)? {
                cached_rules.push(rule);
            } else {
                rules.push( create_rule_for_type(&data_source, i, variants_count)? );
            }
        }
        let data = data_source.get_result_datas()?;
        if variants_count != data.len() {
            return Err(format!("Error in DecisionTable data! Found {} variants count, where required {}",
                               data.len(), variants_count));
        }
        Ok(DecisionTable {
            rules,
            cached_rules, 
            data,
            variants_count, 
        })
    }
    
    pub fn check_all_cached(&mut self, value: &JsonValue) -> Result<Vec<Cow<String>>, String> {
        let mut indices: Option<Cow<Vec<usize>>> = None;
        for mut rule in &mut self.cached_rules { 
            let match_res = rule.check_all_cached(value);
            indices = match match_res {
                Ok(indices_new) => {
                    match indices {
                        Some(indices_old) => 
                            match indices_old {
                                Cow::Borrowed(indices_old_ref) => Some(Cow::Owned(indices_old_ref.into_iter()
                                    .filter(|x| indices_new.contains(x))
                                    .map(|x| { *x }).collect())),
                                Cow::Owned(indices_old) => Some(Cow::Owned(indices_old.into_iter().filter(|x| indices_new.contains(x)).collect())),
                            },
                        None => Some(Cow::Borrowed(indices_new)),
                    }
                },
                Err(e) => return Err(e),
            };
        }
        let mut indices = indices.ok_or("Error in DecisionTable data! No rules found".to_string())?
            .into_owned();
        for rule in &self.rules {
            match rule.check_indices(value, &indices) {
                Ok(indices_new) => {
                    indices = indices_new;
                },
                Err(e) => return Err(e)
            }
        }

        Ok(self.map_indices_to_values(&indices)?)
    }
    
    pub fn check_all(&self, value: &JsonValue) -> Result<Vec<Cow<String>>, String> {
        let mut indices: Vec<usize> = (0..self.variants_count).collect();
        for rule in &self.rules {
            match rule.check_indices(value, &indices) {
                Ok(indices_new) => {
                    indices = indices_new;
                },
                Err(e) => return Err(e)
            }
        }
        for rule in &self.cached_rules {
            match rule.check_indices(value, &indices) {
                Ok(indices_new) => {
                    indices = indices_new;
                },
                Err(e) => return Err(e)
            }
        }

        Ok(self.map_indices_to_values(&indices)?)
    }
    


    pub fn check_all_parallel(&self, value: &JsonValue) -> Result<Vec<Cow<String>>, String> {
        let mut indices: Vec<usize> = (0..self.variants_count).collect();
        for rule in &self.rules {
            let match_res = if indices.len() > 2000 {
                let thread_count = (indices.len() / 4000) as u8;
                rule.check_all_multi_thread(value, &indices, thread_count)
            } else {
                rule.check_indices(value, &indices)
            };
            indices = match match_res {
                Ok(indices_new) => indices_new,
                Err(e) => return Err(e),
            };
        }
        for rule in &self.cached_rules {
            let match_res = if indices.len() > 2000 {
                let thread_count = (indices.len() / 4000) as u8;
                rule.check_all_multi_thread(value, &indices, thread_count)
            } else {
                rule.check_indices(value, &indices)
            };
            indices = match match_res {
                Ok(indices_new) => indices_new,
                Err(e) => return Err(e),
            };
        }

        Ok(self.map_indices_to_values(&indices)?)
    }

    fn map_indices_to_values(&self, indices: &Vec<usize>) -> Result<Vec<Cow<String>>, String> {
        let mut result: Vec<Cow<String>> = Vec::new();
        for i in indices {
            result.push(self.data.item(*i)
                .ok_or(format!("Error in DecisionTable data! Index out of bounds: {}", i))?);
        }
        Ok(result)
    }
    
    
}

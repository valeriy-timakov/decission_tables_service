extern crate core;

mod json_parseable;
mod conditions;

use std::borrow::Cow;
use std::collections::HashMap;
use std::env::var;
use std::fmt::format;
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;
use json_parseable::JsonParseable;
use conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};
use calamine::{Reader, open_workbook, Xlsx, DataType, Range, Sheet};
use std::path::Path;
use std::f64;
use std::iter::Map;
use chrono::{DateTime, Utc};

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

enum DecimalType {
    F32,
    F64,
}

struct IntegerData {
    has_negative: bool,
    max_value: i128
}

struct StringData {
    max_length: usize,
    contains_utf: bool,
}

enum NumberType {
    Integer(IntegerData),
    Decimal(DecimalType),
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
    Number(NumberType),
    DateTime,
    Boolean,
}

struct RuleData {
    field_name: String,
    condition: ValueCondition,
    field_type: ValueType,
}

trait DecisionTableSource {
    fn init(&mut self) -> Result<(), String>;
    fn get_rules_count(&self) -> Result<usize, String>;
    fn get_variants_count(&self) -> Result<usize, String>;
    fn get_rule_data(&self, rule_num: usize) -> Result<&RuleData, String>;
    fn get_result_datas(&self) -> Result<Vec<String>, String>;
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
    
    fn get_date_time_rule_values(&self, rule_num: usize) -> Vec<Option<DateTime<Utc>>>;
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

fn create_rule_for_type(data_source: &Box<dyn DecisionTableSource>, rule_num: usize) -> Result<Box<dyn RuleTrait>, String> {
    let rule_data: &RuleData = data_source.get_rule_data(rule_num)?;
    match &rule_data.field_type {  
        ValueType::String(type_data) => Ok(Box::new(create_rule(rule_data, data_source.get_string_rule_values(rule_num)))),
        ValueType::Boolean => Ok(Box::new(create_rule(rule_data, data_source.get_bool_rule_values(rule_num)))),
        ValueType::DateTime => Ok(Box::new(create_rule(rule_data, data_source.get_date_time_rule_values(rule_num)))),
        ValueType::Number(number_type) => {
            match number_type {
                NumberType::Decimal(decimal_type) => {
                    match decimal_type {
                        DecimalType::F64 => Ok(Box::new(create_rule(rule_data, data_source.get_f64_rule_values(rule_num)))),
                        DecimalType::F32 => Ok(Box::new(create_rule(rule_data, data_source.get_f32_rule_values(rule_num)))),
                    }
                },
                NumberType::Integer(integer_data) => {
                    if integer_data.has_negative {
                        if integer_data.max_value <= i8::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i8_rule_values(rule_num))))
                        } else if integer_data.max_value <= i16::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i16_rule_values(rule_num))))
                        } else if integer_data.max_value <= i32::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i32_rule_values(rule_num))))
                        } else {
                            Ok(Box::new(create_rule(rule_data, data_source.get_i64_rule_values(rule_num))))
                        }
                    } else {
                        if integer_data.max_value <= u8::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u8_rule_values(rule_num))))
                        } else if integer_data.max_value <= u16::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u16_rule_values(rule_num))))
                        } else if integer_data.max_value <= u32::MAX as i128 {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u32_rule_values(rule_num))))
                        } else {
                            Ok(Box::new(create_rule(rule_data, data_source.get_u64_rule_values(rule_num))))
                        }
                    }
                }
            }
        }
    }
}

impl DecisionTable {
    fn create(mut data_source: Box<dyn DecisionTableSource>) -> Result<DecisionTable, String> {
        let vaiants_count = data_source.get_rules_count()?;
        let rules_count = data_source.get_rules_count()?;
        let mut rules: Vec<Box<dyn RuleTrait>> = Vec::new();
        for i in 0..rules_count {
            rules.push( create_rule_for_type(&data_source, i)? );
        }
        let data = data_source.get_result_datas()?;
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

struct ParsePreferences {
    decimal_separator: char,
    wildcard: DataType,
}

struct XlsxDTDataSource {
    file_path: String,
    parse_preferences: ParsePreferences, 
    opened_sheet: Option<Range<DataType>>,
    rules_data: Option<HashMap<usize, RuleData>>, 
}

impl XlsxDTDataSource {
    
    const FIELD_NAME_ROW: usize = 0;
    const FIELD_TYPE_ROW: usize = 1;
    const FIELD_CONDITION_ROW: usize = 2;
    const DATA_ROWS_START: usize = 3;
    pub fn new(file_path: String, parse_preferences: ParsePreferences) -> Self {
        XlsxDTDataSource { 
            file_path, 
            parse_preferences,
            opened_sheet: None, 
            rules_data: None,
        }
    }

    fn parse_value_type(&self, sheet: &Range<DataType>, col_num: usize) -> Result<ValueType, String> {
        let type_str_raw = sheet.get((XlsxDTDataSource::FIELD_TYPE_ROW, col_num))
            .ok_or(format!("No type found for column {}", col_num))? ;
        let type_str: &str = match type_str_raw {
                DataType::String(s) => s,
                other => return Err(format!("Invalid type data in column {}! Found: {}", col_num, other))
            };
        match type_str {
            "String" => Ok(ValueType::String(self.look_for_string_data(sheet, col_num)?)),
            "Number" => Ok(ValueType::Number(self.look_for_number_type(sheet, col_num)?)),
            "DateTime" => Ok(ValueType::DateTime),
            "Boolean" => Ok(ValueType::Boolean),
            _ => Err(format!("Unknown type: {}", type_str))
        }
    }

    fn parse_value_condition(&self, sheet: &Range<DataType>, col_num: usize) -> Result<ValueCondition, String> {
        let condition_str = sheet.get((XlsxDTDataSource::FIELD_CONDITION_ROW, col_num))
            .ok_or(format!("No condition found for column {}", col_num))?;
        if let DataType::String(s) = condition_str {
            match s.as_str() {
                "<" => Ok(ValueCondition::LessThan),
                ">" => Ok(ValueCondition::GreaterThan),
                "<=" => Ok(ValueCondition::LessOrEqual),
                ">=" => Ok(ValueCondition::GreaterOrEqual),
                "=" => Ok(ValueCondition::Equal),
                _ => Err(format!("Unknown condition: {}", s))
            }
        } else {
            Err(format!("Invalid condition data in column {}! Found: {}", col_num, condition_str))
        }
    }

    fn look_for_string_data(&self, sheet: &Range<DataType>, col_num: usize) -> Result<StringData, String> {
        let (height, width) = sheet.get_size();
        if col_num >= width {
            return Err(format!("Column number out of range: {} >= {}", col_num, width));
        }
        let mut max_length = 0;
        let mut contains_utf = false;
        for row_index in XlsxDTDataSource::DATA_ROWS_START..height {
            if let Some(cell) = sheet.get((row_index, col_num)) {
                if let DataType::String(s) = cell {
                    max_length = max_length.max(s.len());
                    if s.chars().any(|c| c as u32 > 127) {
                        contains_utf = true;
                    }
                } else {
                    return Err(format!("Wrong cell value for String: {}! In cell [{}, {}]",
                                        cell, col_num, row_index));
                }
            } else {
                return Err(format!("Empty cell in column {} at row {}", col_num, row_index));
            }
        }
        Ok(StringData { max_length, contains_utf })
    }
    
    fn look_for_number_type(&self, sheet: &Range<DataType>, col_num: usize) -> Result<NumberType, String> {
        struct RuleRowsStatistics<'a> {
            parse_preferences: &'a ParsePreferences, 
            contains_decimal: bool,
            contains_f64: bool,
            contains_negative: bool,
            max_value: i128,
        }
        
        impl<'a> RuleRowsStatistics<'a> {
            fn new(parse_preferences: &'a ParsePreferences) -> Self {
                RuleRowsStatistics {
                    parse_preferences, 
                    contains_decimal: false,
                    contains_f64: false,
                    contains_negative: false,
                    max_value: 0,
                }
            }

            fn check_int(&mut self, value_ref: &i64) {
                let value = *value_ref;
                if value < 0_i64 {
                    self.contains_negative = true;
                }
                self.max_value = self.max_value.max(value as i128);
            }

            fn check_float(&mut self, value_ref: &f64) {
                let value = *value_ref;
                if value.fract() != 0.0 {
                    self.contains_decimal = true;
                    if !XlsxDTDataSource::can_fit_in_f32(value) {
                        self.contains_f64 = true;
                    }
                } else {
                    self.check_int(&(value as i64));
                }
            }

            fn check_string(&mut self, s: &str) {

                let s: Cow<str> = if self.parse_preferences.decimal_separator != '.' {
                    Cow::Owned(s.replace(self.parse_preferences.decimal_separator, "."))
                } else {
                    Cow::Borrowed(s)
                };

                if let Ok(value) = s.parse::<i64>() {
                    self.check_int(&value);
                } else if let Ok(value) = s.parse::<f64>() {
                    self.check_float(&value);
                }
            }
        }
        
        let (height, width) = sheet.get_size();
        
        if col_num >= width {
            return Err(format!("Column number out of range: {} >= {}", col_num, width));
        }

        let mut statistics = RuleRowsStatistics::new(&self.parse_preferences);

        for row_index in XlsxDTDataSource::DATA_ROWS_START..height {
            match sheet.get((row_index, col_num)) {
                Some(cell) => {
                    // Check if the cell value is not a number and return an error
                    match cell {
                        DataType::Int(value_ref) => statistics.check_int(value_ref),
                        DataType::Float(value_reff) => statistics.check_float(value_reff),
                        DataType::String(s) => statistics.check_string(s),
                        other => return Err(format!("Wrong cell value for Number: {}! In cell [{}, {}]",
                                                    other, col_num, row_index))
                    }
                }, 
                None => return Err(format!("Empty cell in column {} at row {}", col_num, row_index))
            }
        }

        let res = 
            if statistics.contains_decimal {
                if statistics.contains_f64 {
                    NumberType::Decimal(DecimalType::F64)
                } else {
                    NumberType::Decimal(DecimalType::F32)
                }
            } else {
                NumberType::Integer(IntegerData { 
                    has_negative: statistics.contains_negative, 
                    max_value: statistics.max_value 
                })
            };
        Ok(res)
    }
    
    fn can_fit_in_f32(x: f64) -> bool {
        let x_f32 = x as f32;
        let x_f64 = x_f32 as f64;
        x == x_f64
    }
    

    fn parse_cell_value<T: std::str::FromStr>(&self, cell: &DataType) -> Option<T> {
        match cell {
            DataType::String(s) if s == "*" => None,
            DataType::String(s) => s.parse::<T>().ok(),
            DataType::Int(i) => i.to_string().parse::<T>().ok(),
            DataType::Float(f) => f.to_string().parse::<T>().ok(),
            DataType::Bool(b) => b.to_string().parse::<T>().ok(),
            _ => None,
        }
    }

    fn get_result_datas<T, F: Fn(DataType) -> T>(&self, col_num: usize, mapper: F) -> Result<Vec<T>, String> {
        let sheet = self.opened_sheet.as_ref()
            .ok_or("Data source not inited!")?;

        if col_num >= sheet.width() {
            return Err(format!("Column number out of range: {} >= {}", col_num, sheet.width()));
        }
        
        let res = (XlsxDTDataSource::DATA_ROWS_START..sheet.height())
            .map(|row| sheet.get((row, col_num)).map(mapper)
                .ok_or(format!("Failed to get cell [{}, {}]", row, col_num)))
            .collect();
        Ok(res)
    }
}

impl DecisionTableSource for XlsxDTDataSource {
    fn init(&mut self) -> Result<(), String> {
        let path = Path::new(&self.file_path);
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| format!("Failed to open workbook: {}", e))?;
        
        let sheet = workbook.worksheet_range_at(0)
            .ok_or("No worksheet found")?
            .map_err(|e| {format!("Failed to open workbook: {}", e)})?;
        
        self.opened_sheet = Some(sheet);
        
        let sheet = self.opened_sheet.as_ref().unwrap();

        if sheet.height() < XlsxDTDataSource::DATA_ROWS_START + 1 {
            return Err("Table must have at least 4 rows".to_string());
        }

        // Перевірка останнього стовпця на "out"
        let field_names = sheet.rows().next()
            .ok_or("Empty worksheet")?;
        if let Some(last_field) = field_names.last() {
            if last_field.to_string() != "out" {
                return Err("'Return value' column must be named 'out'".to_string());
            }
        }

        let (height, width) = sheet.get_size();
        let rows_count = height - XlsxDTDataSource::DATA_ROWS_START;
        let rules_count = width - 1;

        let mut rules_data: HashMap<usize, RuleData> = HashMap::new();
        for i in 0..rules_count {
            let field_name = match sheet.get((XlsxDTDataSource::FIELD_NAME_ROW, i)) {
                Some(DataType::String(s)) => s.to_string(),
                _ => return Err(format!("No field name found for column {}!", i))
            };
            let field_type = self.parse_value_type(&sheet, i)
                .map_err(|e| format!("Failed to parse value type: {}", e))?;
            let condition = self.parse_value_condition(&sheet, i)
                .map_err(|e| format!("Failed to parse value condition: {}", e))?;
            let rule_data = RuleData {
                field_name,
                condition,
                field_type,
            };
            rules_data.insert(i, rule_data);
        }
        self.rules_data = Some(rules_data);

        Ok(())
    }

    fn get_rules_count(&self) -> Result<usize, String> {
        self.rules_data.as_ref()
            .map(|data| data.len())
            .ok_or("Data source not inited!".to_string())
    }

    fn get_variants_count(&self) -> Result<usize, String> {
        self.opened_sheet.as_ref()
            .map(|sheet| sheet.height() - XlsxDTDataSource::DATA_ROWS_START)
            .ok_or("Data source not inited!".to_string())
    }

    fn get_rule_data(&self, rule_num: usize) -> Result<&RuleData, String> {
        self.rules_data.as_ref()
            .ok_or("Data source not inited!".to_string())?
            .get(&rule_num)
            .ok_or(format!("No rule data found for rule number {}!", rule_num))
    }

    fn get_result_datas(&self) -> Result<Vec<String>, String> {
        let sheet = self.opened_sheet.as_ref()
            .ok_or("Data source not inited!")?;
        
        let last_col = sheet.width() - 1;
        let res = (XlsxDTDataSource::DATA_ROWS_START..sheet.height())
            .map(|row| sheet.get((row, last_col)).unwrap().to_string())
            .collect();
        Ok(res)
    }

    // Реалізація для кожного типу даних
    fn get_u8_rule_values(&self, rule_num: usize) -> Vec<Option<u8>> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        (3..sheet.height())
            .map(|row| self.parse_cell_value(sheet.get((row, rule_num)).unwrap()))
            .collect()
    }

    // Аналогічно реалізуємо інші методи для різних типів...
    fn get_u16_rule_values(&self, rule_num: usize) -> Vec<Option<u16>> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        (3..sheet.height())
            .map(|row| self.parse_cell_value(sheet.get((row, rule_num)).unwrap()))
            .collect()
    }

    fn get_u32_rule_values(&self, rule_num: usize) -> Vec<Option<u32>> {
        todo!()
    }

    fn get_u64_rule_values(&self, rule_num: usize) -> Vec<Option<u64>> {
        todo!()
    }

    fn get_i8_rule_values(&self, rule_num: usize) -> Vec<Option<i8>> {
        todo!()
    }

    fn get_i16_rule_values(&self, rule_num: usize) -> Vec<Option<i16>> {
        todo!()
    }

    fn get_i32_rule_values(&self, rule_num: usize) -> Vec<Option<i32>> {
        todo!()
    }

    fn get_i64_rule_values(&self, rule_num: usize) -> Vec<Option<i64>> {
        todo!()
    }

    fn get_f32_rule_values(&self, rule_num: usize) -> Vec<Option<f32>> {
        todo!()
    }

    fn get_f64_rule_values(&self, rule_num: usize) -> Vec<Option<f64>> {
        todo!()
    }

    fn get_string_rule_values(&self, rule_num: usize) -> Vec<Option<String>> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        (3..sheet.height())
            .map(|row| {
                match sheet.get((row, rule_num)).unwrap() {
                    DataType::String(s) if s == "*" => None,
                    DataType::String(s) => Some(s.to_string()),
                    _ => None,
                }
            })
            .collect()
    }

    fn get_bool_rule_values(&self, rule_num: usize) -> Vec<Option<bool>> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        (3..sheet.height())
            .map(|row| self.parse_cell_value(sheet.get((row, rule_num)).unwrap()))
            .collect()
    }

    fn get_date_time_rule_values(&self, rule_num: usize) -> Vec<Option<DateTime<Utc>>> {
        todo!()
    }
}
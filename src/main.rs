extern crate core;

mod json_parseable;
mod conditions;

use std::env::var;
use std::fmt::format;
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;
use json_parseable::JsonParseable;
use conditions::{Condition, EqualCondition, GreaterThanCondition, GreaterThanOrEqualCondition, LessThanCondition, LessThanOrEqualCondition};
use calamine::{Reader, open_workbook, Xlsx, DataType, Range};
use std::path::Path;
use std::f64;

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
        ValueType::Number(number_type) => {
            match number_type {
                NumberType::Decimal(decimal_type) => {
                    match decimal_type {
                        DecimalType::F64 => Box::new(create_rule(rule_data, data_source.get_f64_rule_values(rule_num))),
                        DecimalType::F32 => Box::new(create_rule(rule_data, data_source.get_f32_rule_values(rule_num))),
                    }
                },
                NumberType::Integer(integer_data) => {
                    if integer_data.has_negative {
                        if integer_data.max_value <= i8::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_i8_rule_values(rule_num)))
                        } else if integer_data.max_value <= i16::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_i16_rule_values(rule_num)))
                        } else if integer_data.max_value <= i32::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_i32_rule_values(rule_num)))
                        } else {
                            Box::new(create_rule(rule_data, data_source.get_i64_rule_values(rule_num)))
                        }
                    } else {
                        if integer_data.max_value <= u8::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_u8_rule_values(rule_num)))
                        } else if integer_data.max_value <= u16::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_u16_rule_values(rule_num)))
                        } else if integer_data.max_value <= u32::MAX as i128 {
                            Box::new(create_rule(rule_data, data_source.get_u32_rule_values(rule_num)))
                        } else {
                            Box::new(create_rule(rule_data, data_source.get_u64_rule_values(rule_num)))
                        }
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

struct XlsxDTDataSource {
    file_path: String,
    rule_data: Option<RuleData>, 
}

impl XlsxDTDataSource {
    
    const FIELD_NAME_ROW: usize = 0;
    const FIELD_TYPE_ROW: usize = 1;
    const FIELD_CONDITION_ROW: usize = 2;
    const DATA_ROWS_START: usize = 3;
    pub fn new(file_path: String) -> Self {
        XlsxDTDataSource { 
            file_path,
            rule_data: None,
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
            "String" => {
                Ok(ValueType::String(StringData { max_length: 0, contains_utf: false }))
            },
            "Number" => Ok(ValueType::Number(XlsxDTDataSource::look_for_number_type(sheet, col_num)?)),
            "Boolean" => Ok(ValueType::Boolean),
            _ => Err(format!("Unknown type: {}", type_str))
        }
    }

    fn parse_value_condition(&self, condition_str: &str) -> Result<ValueCondition, String> {
        match condition_str {
            "<" => Ok(ValueCondition::LessThan),
            ">" => Ok(ValueCondition::GreaterThan),
            "<=" => Ok(ValueCondition::LessOrEqual),
            ">=" => Ok(ValueCondition::GreaterOrEqual),
            "=" => Ok(ValueCondition::Equal),
            _ => Err(format!("Unknown condition: {}", condition_str))
        }
    }
    
    fn look_for_number_type(sheet: &Range<DataType>, col_num: usize) -> Result<NumberType, String> {
        let mut max_value: Option<f64> = None;

        // Отримуємо розміри таблиці
        let (height, width) = sheet.get_size();
        
        if col_num >= width {
            return Err(format!("Column number out of range: {} >= {}", col_num, width));
        }

        let mut contains_decimal = false;
        let mut contains_f64 = false;
        let mut contains_negative = false;
        let mut max_value: i128 = 0;

        for row_index in 0..height {
            if let Some(cell) = sheet.get((row_index, col_num)) {
                // Check if the cell value is not a number and return an error
                match cell {
                    DataType::Int(value_ref) => {
                        let value = *value_ref;
                        if value < 0_i64 {
                            contains_negative = true;
                        }
                        max_value = max_value.max(value as i128);
                    },
                    DataType::Float(value_reff) => {
                        let value = *value_reff;
                        if value.fract() == 0.0 {
                            contains_decimal = true;
                        }
                    },
                    DataType::String(s) => s.parse::<f64>().ok(),
                    _ => None, // Non-numeric data types
                };

            }
        }

        Ok(max_value)
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
}

impl DecisionTableSource for XlsxDTDataSource {
    fn init(&mut self) -> Result<(), String> {
        let path = Path::new(&self.file_path);
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| format!("Failed to open workbook: {}", e))?;
        
        let sheet = workbook.worksheet_range_at(0)
            .ok_or("No worksheet found")?
            .map_err(|e| {format!("Failed to open workbook: {}", e)})?;

        if sheet.height() < 4 {
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

        Ok(())
    }

    fn get_rules_count(&self) -> usize {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        sheet.width() - 1 // Віднімаємо стовпець з результатами
    }

    fn get_variants_count(&self) -> usize {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        sheet.height() - 3 // Віднімаємо заголовок, тип та умову
    }

    fn get_rule_data(&self, rule_num: usize) -> &RuleData {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        let field_name = sheet.get((0, rule_num)).unwrap().to_string();
        let type_str = sheet.get((1, rule_num)).unwrap().to_string();
        let condition_str = sheet.get((2, rule_num)).unwrap().to_string();
        
        let field_type = self.parse_value_type(&type_str).unwrap();
        let condition = self.parse_value_condition(&condition_str).unwrap();
        
        Box::leak(Box::new(RuleData {
            field_name,
            condition,
            field_type,
        }))
    }

    fn get_result_datas(&self) -> Vec<String> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path).unwrap();
        let sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
        
        let last_col = sheet.width() - 1;
        (3..sheet.height())
            .map(|row| sheet.get((row, last_col)).unwrap().to_string())
            .collect()
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

    // ... (аналогічно для інших числових типів)

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
}
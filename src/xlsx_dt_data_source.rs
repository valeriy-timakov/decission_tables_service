
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::var;
use std::fmt::format;
use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;
use calamine::{Reader, open_workbook, Xlsx, DataType, Range, Sheet};
use std::path::Path;
use std::f64;
use crate::decession_table::{DecisionTableSource, RuleData, ValueType, ValueCondition, StringData, NumberType, DecimalType, IntegerData};
use std::iter::Map;
use chrono::{naive, DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use core::str::FromStr;

pub struct ParsePreferences {
    decimal_separator: char,
    wildcard: DataType,
    date_time_format: String,
}

impl ParsePreferences {
    pub fn new(decimal_separator: char, wildcard: DataType, date_time_format: String) -> Self {
        ParsePreferences {
            decimal_separator,
            wildcard,
            date_time_format,
        }
    }
}

pub struct XlsxDTDataSource {
    file_path: String,
    parse_preferences: ParsePreferences,
    opened_sheet: Option<Range<DataType>>,
    rules_data: Option<HashMap<usize, RuleData>>,
}

macro_rules! generate_get_rule_values_fn {
    ($name:ident, $type:ty) => {
        fn $name(&self, rule_num: usize) -> Result<Vec<Option<$type>>, String> {
            self.get_result_datas(rule_num, |cell| {
                let res = match cell {
                    DataType::String(s) => s.parse::<$type>()
                        .map_err(|e| format!("Failed to parse cell value: {}", e))?,
                    DataType::Int(i) => *i as $type,
                    DataType::Float(f) => *f as $type,
                    _ => return Err(format!("Invalid cell type: {:?}", cell))
                };
                Ok(res)
            })
        }
    };
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
        const DATETIME_PREFIX: &str = "DateTime";
        if type_str.starts_with(DATETIME_PREFIX) {
            if type_str.chars().nth(DATETIME_PREFIX.len()) == Some('(') && type_str.ends_with(')') {
                let time_zone_raw = &type_str[DATETIME_PREFIX.len() + 1..type_str.len() - 1];
                let offset = Self::parse_time_zone(time_zone_raw)?;
                Ok(ValueType::DateTime(offset))
            } else {
                Err(format!("Invalid DateTime time zone: {}", type_str))
            }
        } else {
            match type_str {
                "String" => Ok(ValueType::String(self.look_for_string_data(sheet, col_num)?)),
                "Number" => Ok(ValueType::Number(self.look_for_number_type(sheet, col_num)?)),
                "Boolean" => Ok(ValueType::Boolean),
                _ => Err(format!("Unknown type: {}", type_str))
            }
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

    fn parse_time_zone(s: &str) -> Result<FixedOffset, String> {
        let (hours_str, minutes_str) = s.split_once(':')
            .or_else(|| Some((s, ""))).unwrap();
        let hours = hours_str.parse::<i32>()
            .map_err(|e| { format!("Error parsing timezone hours! Cause: {}", e) })?;
        let minutes = minutes_str.parse::<i32>()
            .map_err(|e| { format!("Error parsing timezone hours! Cause: {}", e) })?;
        FixedOffset::east_opt(hours * 3600 + minutes * 60)
            .ok_or(format!("Invalid time zone! Found: {}", s))
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
        Ok(StringData::new(max_length, contains_utf))
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
                NumberType::Integer(IntegerData::new(statistics.contains_negative, statistics.max_value))
            };
        Ok(res)
    }

    fn can_fit_in_f32(x: f64) -> bool {
        let x_f32 = x as f32;
        let x_f64 = x_f32 as f64;
        x == x_f64
    }
    
    fn get_result_datas<T, F: Fn(&DataType) -> Result<T, String>>(&self, col_num: usize, mapper: F) -> Result<Vec<Option<T>>, String> {
        let sheet = self.opened_sheet.as_ref()
            .ok_or("Data source not initialized!")?;

        if col_num >= sheet.width() {
            return Err(format!("Column number out of range: {} >= {}", col_num, sheet.width()));
        }

        let mut results = Vec::with_capacity(sheet.height() - XlsxDTDataSource::DATA_ROWS_START);
        for row in XlsxDTDataSource::DATA_ROWS_START..sheet.height() {
            match sheet.get((row, col_num)) {
                Some(cell) => {
                    if *cell == self.parse_preferences.wildcard {
                        results.push(None);
                    } else {
                        match mapper(cell) {
                            Ok(mapped_value) => results.push(Some(mapped_value)),
                            Err(err) => return Err(err),
                        }
                    }
                }
                None => return Err(format!("Failed to get cell [{}, {}]", row, col_num)),
            }
        }
        Ok(results)
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
            let rule_data = RuleData::new(field_name, condition, field_type);
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

    generate_get_rule_values_fn!(get_u8_rule_values, u8);
    generate_get_rule_values_fn!(get_u16_rule_values, u16);
    generate_get_rule_values_fn!(get_u32_rule_values, u32);
    generate_get_rule_values_fn!(get_u64_rule_values, u64);
    generate_get_rule_values_fn!(get_i8_rule_values, i8);
    generate_get_rule_values_fn!(get_i16_rule_values, i16);
    generate_get_rule_values_fn!(get_i32_rule_values, i32);
    generate_get_rule_values_fn!(get_i64_rule_values, i64);
    generate_get_rule_values_fn!(get_f32_rule_values, f32);
    generate_get_rule_values_fn!(get_f64_rule_values, f64);

    fn get_string_rule_values(&self, rule_num: usize) -> Result<Vec<Option<String>>, String> {
        self.get_result_datas(rule_num, |cell| {
            match cell {
                DataType::String(s) => Ok(s.to_string()),
                _ => Err(format!("Invalid cell type: {:?}", cell))
            }
        })
    }

    fn get_bool_rule_values(&self, rule_num: usize) -> Result<Vec<Option<bool>>, String> {
        self.get_result_datas(rule_num, |cell| {
            match cell {
                DataType::String(s) => bool::from_str(s)
                    .map_err(|e| format!("Failed to parse cell value: {}", e)),
                DataType::Bool(b) => Ok(*b),
                _ => Err(format!("Invalid cell type: {:?}", cell))
            }
        })
    }

    fn get_date_time_rule_values(&self, rule_num: usize) -> Result<Vec<Option<DateTime<FixedOffset>>>, String> {

        fn parse_datetime(cell: &DataType, time_zone: &FixedOffset) -> Result<DateTime<FixedOffset>, String> {
            let naive_date_time = cell.as_datetime()
                .ok_or("Failed to parse DateTime cell value".to_string())?;
            time_zone.from_local_datetime(&naive_date_time)
                .single()
                .ok_or("Failed to convert to UTC".to_string())
        }

        let dt_format = &self.parse_preferences.date_time_format;
        let rule_type = &self.get_rule_data(rule_num)?.field_type;
        let time_zone = match rule_type { 
            ValueType::DateTime(offset) => offset,
            _ => return Err("Invalid rule type".to_string())
        };
        
        self.get_result_datas(rule_num, |cell| {
            match cell {
                DataType::String(s) => NaiveDateTime::parse_from_str(s, dt_format)
                    .map_err(|e| format!("Failed to parse cell value: {}", e))
                    .and_then(|x| time_zone.from_local_datetime(&x).single()
                        .ok_or(format!("Date time {} is ambigous!", s)))
                    ,
                DataType::DateTime(_) => parse_datetime(cell, time_zone),
                DataType::DateTimeIso(_) => parse_datetime(cell, time_zone),
                _ => Err(format!("Invalid cell type: {:?}", cell))
            }
        })
    }
}
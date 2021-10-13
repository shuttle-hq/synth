use crate::Value;
use serde_json::Map;
use std::collections::BTreeMap;
use synth_gen::value::Number;

pub fn synth_val_to_json(val: Value) -> serde_json::Value {
    match val {
        Value::Null(_) => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => serde_json::Value::Number(synth_num_to_json(n)),
        Value::String(s) => serde_json::Value::String(s),
        Value::DateTime(dt) => serde_json::Value::String(dt.format_to_string()),
        Value::Object(obj) => serde_json::Value::Object(synth_obj_to_json(obj)),
        Value::Array(arr) => serde_json::Value::Array(synth_arr_to_json(arr)),
    }
}

fn synth_num_to_json(n: Number) -> serde_json::Number {
    match n {
        Number::I8(i8) => serde_json::Number::from(i8),
        Number::I16(i16) => serde_json::Number::from(i16),
        Number::I32(i32) => serde_json::Number::from(i32),
        Number::I64(i64) => serde_json::Number::from(i64),
        Number::I128(i128) => serde_json::Number::from(i128 as i64),
        Number::U8(u8) => serde_json::Number::from(u8),
        Number::U16(u16) => serde_json::Number::from(u16),
        Number::U32(u32) => serde_json::Number::from(u32),
        Number::U64(u64) => serde_json::Number::from(u64),
        Number::U128(u128) => serde_json::Number::from(u128 as u64),
        Number::F32(f32) => serde_json::Number::from_f64(*f32 as f64)
            .unwrap_or_else(|| panic!("Could not convert value '{}' to JSON f64", f32)),
        Number::F64(f64) => serde_json::Number::from_f64(*f64)
            .unwrap_or_else(|| panic!("Could not convert value '{}' to JSON f64", f64)),
    }
}

fn synth_obj_to_json(obj: BTreeMap<String, Value>) -> Map<String, serde_json::Value> {
    obj.into_iter()
        .map(|(k, v)| (k, synth_val_to_json(v)))
        .collect()
}

fn synth_arr_to_json(arr: Vec<Value>) -> Vec<serde_json::Value> {
    arr.into_iter().map(synth_val_to_json).collect()
}

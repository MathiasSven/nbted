use crate::Result;
use crate::data::{Compression, NBT, NBTFile};

use serde_json::Value;
use std::io::Read;

use anyhow::{Context, bail};

/// Read an NBT file from JSON format
pub fn read_file<R: Read>(reader: &mut R) -> Result<NBTFile> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let json: Value = serde_json::from_slice(&buf)
        .context("Failed to parse JSON")?;

    // Extract compression
    let compression_str = json.get("compression")
        .and_then(|v| v.as_str())
        .unwrap_or("None");

    let compression = Compression::from_str(compression_str)
        .context("Invalid or unknown compression type")?;

    // Extract data
    let root = json.get("data")
        .context("Missing 'data' field in JSON")?;

    let nbt_root = json_to_nbt(root)?;

    Ok(NBTFile { root: nbt_root, compression })
}

/// Convert a JSON value to an NBT tag
fn json_to_nbt(value: &Value) -> Result<NBT> {
    match value {
        Value::Null => Ok(NBT::End),
        Value::Object(obj) => {
            let type_str = obj.get("type")
                .and_then(|v| v.as_str())
                .context("Missing 'type' field in JSON object")?;

            let val_field = obj.get("value")
                .context("Missing 'value' field in JSON object")?;

            match type_str {
                "byte" => {
                    let v = val_field.as_i64()
                        .context("Invalid byte value")?;
                    Ok(NBT::Byte(v as i8))
                }
                "short" => {
                    let v = val_field.as_i64()
                        .context("Invalid short value")?;
                    Ok(NBT::Short(v as i16))
                }
                "int" => {
                    let v = val_field.as_i64()
                        .context("Invalid int value")?;
                    Ok(NBT::Int(v as i32))
                }
                "long" => {
                    let v = val_field.as_str()
                        .context("Invalid long value")?
                        .parse::<i64>()
                        .context("Failed to parse long value")?;
                    Ok(NBT::Long(v))
                }
                "float" => {
                    let v = val_field.as_f64()
                        .context("Invalid float value")?;
                    Ok(NBT::Float(v as f32))
                }
                "double" => {
                    let v = val_field.as_f64()
                        .context("Invalid double value")?;
                    Ok(NBT::Double(v))
                }
                "byte_array" => {
                    let arr = val_field.as_array()
                        .context("Invalid byte_array value")?;
                    let bytes: Result<Vec<i8>> = arr.iter()
                        .map(|v| v.as_i64()
                            .context("Invalid byte in array")
                            .map(|i| i as i8))
                        .collect();
                    Ok(NBT::ByteArray(bytes?))
                }
                "string" => {
                    let s = val_field.as_str()
                        .context("Invalid string value")?;
                    Ok(NBT::String(s.as_bytes().to_vec()))
                }
                "list" => {
                    let arr = val_field.as_array()
                        .context("Invalid list value")?;
                    let items: Result<Vec<NBT>> = arr.iter()
                        .map(json_to_nbt)
                        .collect();
                    Ok(NBT::List(items?))
                }
                "compound" => {
                    let compound_obj = val_field.as_object()
                        .context("Invalid compound value")?;
                    let mut items = Vec::new();
                    for (key, val) in compound_obj {
                        let nbt = json_to_nbt(val)?;
                        items.push((key.as_bytes().to_vec(), nbt));
                    }
                    Ok(NBT::Compound(items))
                }
                "int_array" => {
                    let arr = val_field.as_array()
                        .context("Invalid int_array value")?;
                    let ints: Result<Vec<i32>> = arr.iter()
                        .map(|v| v.as_i64()
                            .context("Invalid int in array")
                            .map(|i| i as i32))
                        .collect();
                    Ok(NBT::IntArray(ints?))
                }
                "long_array" => {
                    let arr = val_field.as_array()
                        .context("Invalid long_array value")?;
                    let longs: Result<Vec<i64>> = arr.iter()
                        .map(|v| v.as_str()
                            .context("Invalid long in array")
                            .and_then(|s| s.parse::<i64>()
                                .context("Failed to parse long in array")))
                        .collect();
                    Ok(NBT::LongArray(longs?))
                }
                t => bail!("Unknown NBT type: {}", t),
            }
        }
        _ => bail!("Expected JSON object for NBT tag"),
    }
}

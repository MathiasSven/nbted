use crate::Result;
use crate::data::{NBT, NBTFile};

use serde_json::{json, Value};
use std::io::Write;

/// Given an NBT file, write it to the writer in JSON format
pub fn write_file<W: Write>(w: &mut W, file: &NBTFile) -> Result<()> {
    let json_value = nbt_to_json(&file.root)?;
    
    let output = json!({
        "compression": file.compression.to_str(),
        "data": json_value
    });
    
    w.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;
    Ok(())
}

/// Convert an NBT tag to a JSON value
fn nbt_to_json(tag: &NBT) -> Result<Value> {
    match tag {
        NBT::End => Ok(json!(null)),
        NBT::Byte(x) => Ok(json!({
            "type": "byte",
            "value": x
        })),
        NBT::Short(x) => Ok(json!({
            "type": "short",
            "value": x
        })),
        NBT::Int(x) => Ok(json!({
            "type": "int",
            "value": x
        })),
        NBT::Long(x) => Ok(json!({
            "type": "long",
            "value": x.to_string()
        })),
        NBT::Float(x) => Ok(json!({
            "type": "float",
            "value": x
        })),
        NBT::Double(x) => Ok(json!({
            "type": "double",
            "value": x
        })),
        NBT::ByteArray(x) => Ok(json!({
            "type": "byte_array",
            "value": x
        })),
        NBT::String(x) => Ok(json!({
            "type": "string",
            "value": String::from_utf8(x.clone())?
        })),
        NBT::List(x) => {
            let values: Result<Vec<Value>> = x.iter().map(nbt_to_json).collect();
            Ok(json!({
                "type": "list",
                "value": values?
            }))
        }
        NBT::Compound(x) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in x {
                let key_str = String::from_utf8(key.clone())?;
                obj.insert(key_str, nbt_to_json(val)?);
            }
            Ok(json!({
                "type": "compound",
                "value": Value::Object(obj)
            }))
        }
        NBT::IntArray(x) => Ok(json!({
            "type": "int_array",
            "value": x
        })),
        NBT::LongArray(x) => Ok(json!({
            "type": "long_array",
            "value": x.iter().map(|v| v.to_string()).collect::<Vec<_>>()
        })),
    }
}

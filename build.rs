use serde_json::Value;
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=index.json");

    let source = fs::read_to_string("index.json").expect("failed to read index.json");
    let mut document: Value = serde_json::from_str(&source).expect("index.json is not valid JSON");
    convert_openapi_31_to_30(&mut document);

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
    fs::write(
        out_dir.join("index.patched.json"),
        serde_json::to_vec_pretty(&document).expect("failed to serialize patched OpenAPI document"),
    )
    .expect("failed to write patched OpenAPI document");
}

fn convert_openapi_31_to_30(document: &mut Value) {
    if let Value::Object(object) = document {
        object.insert("openapi".into(), Value::String("3.0.3".into()));
        rename_colliding_schemas(document);
        convert_value(document);
    }
}

fn rename_colliding_schemas(document: &mut Value) {
    let renames = [
        ("address", "address-profile"),
        ("address-2", "address-2-profile"),
        ("address-3", "address-3-profile"),
        ("address-4", "address-4-profile"),
    ];

    let mut renamed = Vec::new();
    if let Some(Value::Object(schemas)) = document
        .get_mut("components")
        .and_then(Value::as_object_mut)
        .and_then(|components| components.get_mut("schemas"))
    {
        for (old_name, new_name) in renames {
            if let Some(schema) = schemas.remove(old_name) {
                schemas.insert(new_name.into(), schema);
                renamed.push((old_name, new_name));
            }
        }
    }
    for (old_name, new_name) in renamed {
        rename_references(document, old_name, new_name);
    }
}

fn rename_references(value: &mut Value, old_name: &str, new_name: &str) {
    match value {
        Value::Object(object) => {
            for child in object.values_mut() {
                rename_references(child, old_name, new_name);
            }
        }
        Value::Array(array) => {
            for child in array {
                rename_references(child, old_name, new_name);
            }
        }
        Value::String(reference) if *reference == format!("#/components/schemas/{old_name}") => {
            *reference = format!("#/components/schemas/{new_name}");
        }
        _ => {}
    }
}

fn convert_value(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if let Some(type_value) = object.get_mut("type") {
                if let Value::Array(types) = type_value {
                    let nullable = types.iter().any(|item| item == "null");
                    let non_null_type = types
                        .iter()
                        .find(|item| *item != "null")
                        .cloned()
                        .expect("OpenAPI type union contains only null");
                    *type_value = non_null_type;
                    if nullable {
                        object.insert("nullable".into(), Value::Bool(true));
                    }
                }
            }

            if let Some(exclusive_minimum) = object.get("exclusiveMinimum").cloned() {
                if !exclusive_minimum.is_boolean() {
                    object.insert("minimum".into(), exclusive_minimum);
                    object.insert("exclusiveMinimum".into(), Value::Bool(true));
                }
            }
            if let Some(exclusive_maximum) = object.get("exclusiveMaximum").cloned() {
                if !exclusive_maximum.is_boolean() {
                    object.insert("maximum".into(), exclusive_maximum);
                    object.insert("exclusiveMaximum".into(), Value::Bool(true));
                }
            }

            if let Some(Value::Object(content)) = object.get_mut("content") {
                content.retain(|media_type, media| {
                    !(media_type.starts_with("application/json") && media.get("schema").is_none())
                });

                for media_type in ["application/jose+json", "application/merge-patch+json"] {
                    if let Some(media) = content.remove(media_type) {
                        content.insert("application/json".into(), media);
                    }
                }
                content.remove("multipart/form-data");
            }

            if let Some(Value::Object(responses)) = object.get_mut("responses") {
                for (status, response) in responses {
                    if !status.starts_with('2') {
                        if let Value::Object(response) = response {
                            response.remove("content");
                        }
                    }
                }
            }

            for child in object.values_mut() {
                convert_value(child);
            }
        }
        Value::Array(array) => {
            for child in array {
                convert_value(child);
            }
        }
        Value::String(string) => clean_markdown_directives(string),
        _ => {}
    }
}

fn clean_markdown_directives(string: &mut String) {
    while let Some(start) = string.find("{%") {
        let Some(end_offset) = string[start + 2..].find("%}") else {
            string.truncate(start);
            return;
        };
        string.replace_range(start..start + 2 + end_offset + 2, "");
    }
}

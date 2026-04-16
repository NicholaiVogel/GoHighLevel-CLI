use serde::Serialize;
use serde_json::{Value, json};

use crate::commands::OutputFormat;

#[derive(Debug, Clone, Serialize)]
pub struct ResponseMeta {
    pub schema_version: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuccessEnvelope<T>
where
    T: Serialize,
{
    pub ok: bool,
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEnvelope {
    pub ok: bool,
    pub error: ErrorBody,
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub exit_code: i32,
    pub details: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

pub fn success_envelope<T>(data: T) -> SuccessEnvelope<T>
where
    T: Serialize,
{
    SuccessEnvelope {
        ok: true,
        data,
        meta: meta(),
    }
}

pub fn error_envelope(error: &ghl::GhlError) -> ErrorEnvelope {
    ErrorEnvelope {
        ok: false,
        error: ErrorBody {
            code: error.code().to_owned(),
            message: error.to_string(),
            exit_code: error.exit_code(),
            details: json!({}),
            hint: error.hint().map(str::to_owned),
        },
        meta: meta(),
    }
}

pub fn print_success<T>(data: T, format: OutputFormat, pretty: bool) -> ghl::Result<()>
where
    T: Serialize,
{
    let envelope = success_envelope(data);
    match format {
        OutputFormat::Json => print_json(&envelope, pretty)?,
        OutputFormat::Ndjson => println!("{}", serde_json::to_string(&envelope)?),
        OutputFormat::Table => print_json(&envelope, true)?,
    }
    Ok(())
}

pub fn print_error(error: &ghl::GhlError) {
    let envelope = error_envelope(error);
    let rendered = serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
        r#"{"ok":false,"error":{"code":"general_error","message":"failed to serialize error","exit_code":1,"details":{}},"meta":{"schema_version":"ghl-cli.v1"}}"#.to_owned()
    });
    eprintln!("{rendered}");
}

fn print_json<T>(value: &T, pretty: bool) -> ghl::Result<()>
where
    T: Serialize,
{
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn meta() -> ResponseMeta {
    ResponseMeta {
        schema_version: "ghl-cli.v1",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn error_envelope_contains_stable_shape() {
        let error = ghl::GhlError::Validation {
            message: "invalid thing".to_owned(),
        };
        let envelope = error_envelope(&error);
        let value = serde_json::to_value(envelope).expect("value");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "validation_error");
        assert_eq!(value["error"]["exit_code"], 2);
        assert_eq!(value["meta"]["schema_version"], "ghl-cli.v1");
    }

    #[test]
    fn success_envelope_contains_data_and_meta() {
        let envelope = success_envelope(serde_json::json!({ "hello": "world" }));
        let value: Value = serde_json::to_value(envelope).expect("value");

        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["hello"], "world");
        assert_eq!(value["meta"]["schema_version"], "ghl-cli.v1");
    }
}

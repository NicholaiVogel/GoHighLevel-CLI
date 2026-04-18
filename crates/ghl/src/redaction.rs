use serde_json::Value;

const EXACT_SECRET_KEYS: &[&str] = &[
    "authorization",
    "cookie",
    "token",
    "accesstoken",
    "access_token",
    "refreshtoken",
    "refresh_token",
    "refreshjwt",
    "jwt",
    "apikey",
    "api_key",
    "secret",
    "password",
    "otp",
    "body",
    "message",
    "messagebody",
    "lastmessagebody",
    "html",
    "notes",
];

pub fn redact_header_value(name: &str, value: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if lower == "authorization" || lower == "cookie" || lower.contains("token") {
        "[REDACTED]".to_owned()
    } else {
        redact_token_like(value)
    }
}

pub fn redact_token_like(value: &str) -> String {
    if looks_like_secret(value) {
        "[REDACTED]".to_owned()
    } else {
        value.to_owned()
    }
}

pub fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if key.eq_ignore_ascii_case("notes") && should_preserve_notes_collection(value)
                    {
                        (key.clone(), redact_json(value))
                    } else if is_secret_key(key) {
                        (key.clone(), Value::String("[REDACTED]".to_owned()))
                    } else {
                        (key.clone(), redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_json).collect()),
        Value::String(value) => Value::String(redact_token_like(value)),
        other => other.clone(),
    }
}

fn should_preserve_notes_collection(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.is_empty() || items.iter().all(Value::is_object),
        _ => false,
    }
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    EXACT_SECRET_KEYS
        .iter()
        .any(|secret_key| lower == *secret_key)
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("apikey")
        || lower.contains("api_key")
        || lower.contains("jwt")
}

fn looks_like_secret(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("bearer ")
        || lower.starts_with("pit-")
        || value.matches('.').count() >= 2 && value.len() > 80
        || value.len() > 120
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn authorization_header_is_redacted() {
        assert_eq!(
            redact_header_value("Authorization", "Bearer pit-secret"),
            "[REDACTED]"
        );
    }

    #[test]
    fn token_like_json_fields_are_redacted() {
        let value = json!({
            "token": "pit-secret",
            "nested": { "refreshJwt": "abc.def.ghi" },
            "safe": "hello"
        });
        let redacted = redact_json(&value);

        assert_eq!(redacted["token"], "[REDACTED]");
        assert_eq!(redacted["nested"]["refreshJwt"], "[REDACTED]");
        assert_eq!(redacted["safe"], "hello");
    }

    #[test]
    fn message_collections_are_preserved_while_bodies_are_redacted() {
        let value = json!({
            "messages": [
                {
                    "messageType": "TYPE_SMS",
                    "body": "private customer text"
                }
            ],
            "lastMessageBody": "private preview"
        });
        let redacted = redact_json(&value);

        assert_eq!(redacted["messages"][0]["messageType"], "TYPE_SMS");
        assert_eq!(redacted["messages"][0]["body"], "[REDACTED]");
        assert_eq!(redacted["lastMessageBody"], "[REDACTED]");
    }

    #[test]
    fn note_object_collections_keep_ids_but_redact_body() {
        let value = json!({
            "notes": [
                {
                    "id": "note_123",
                    "body": "private appointment note"
                }
            ]
        });
        let redacted = redact_json(&value);

        assert_eq!(redacted["notes"][0]["id"], "note_123");
        assert_eq!(redacted["notes"][0]["body"], "[REDACTED]");
    }

    #[test]
    fn primitive_notes_arrays_are_redacted() {
        let value = json!({
            "notes": ["private note"]
        });
        let redacted = redact_json(&value);

        assert_eq!(redacted["notes"], "[REDACTED]");
    }
}

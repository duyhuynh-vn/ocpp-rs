//! Message envelope types for OCPP protocol

use crate::{CallErrorCode, MessageType, OcppError, OcppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// OCPP message envelope that wraps all message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// Call message (request)
    Call(CallMessage),
    /// CallResult message (successful response)
    CallResult(CallResultMessage),
    /// CallError message (error response)
    CallError(CallErrorMessage),
}

impl Message {
    /// Get the message type
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Call(_) => MessageType::Call,
            Message::CallResult(_) => MessageType::CallResult,
            Message::CallError(_) => MessageType::CallError,
        }
    }

    /// Get the unique message ID
    pub fn unique_id(&self) -> &str {
        match self {
            Message::Call(msg) => &msg.unique_id,
            Message::CallResult(msg) => &msg.unique_id,
            Message::CallError(msg) => &msg.unique_id,
        }
    }

    /// Create a new Call message
    pub fn call<T>(action: String, payload: T) -> OcppResult<Self>
    where
        T: Serialize,
    {
        Ok(Message::Call(CallMessage {
            message_type: MessageType::Call,
            unique_id: Uuid::new_v4().to_string(),
            action,
            payload: serde_json::to_value(payload)?,
        }))
    }

    /// Create a CallResult message in response to a Call
    pub fn call_result<T>(unique_id: String, payload: T) -> OcppResult<Self>
    where
        T: Serialize,
    {
        Ok(Message::CallResult(CallResultMessage {
            message_type: MessageType::CallResult,
            unique_id,
            payload: serde_json::to_value(payload)?,
        }))
    }

    /// Create a CallError message in response to a Call
    pub fn call_error(
        unique_id: String,
        error_code: CallErrorCode,
        error_description: String,
        error_details: Option<serde_json::Value>,
    ) -> Self {
        Message::CallError(CallErrorMessage {
            message_type: MessageType::CallError,
            unique_id,
            error_code,
            error_description,
            error_details: error_details.unwrap_or(serde_json::Value::Object(Default::default())),
        })
    }
}

/// OCPP Call message (request)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallMessage {
    /// Message type identifier (always 2 for Call)
    #[serde(rename = "0")]
    pub message_type: MessageType,
    /// Unique message identifier
    #[serde(rename = "1")]
    pub unique_id: String,
    /// Action name (e.g., "Authorize", "StartTransaction")
    #[serde(rename = "2")]
    pub action: String,
    /// Message payload
    #[serde(rename = "3")]
    pub payload: serde_json::Value,
}

impl CallMessage {
    /// Create a new Call message
    pub fn new<T>(action: String, payload: T) -> OcppResult<Self>
    where
        T: Serialize,
    {
        Ok(CallMessage {
            message_type: MessageType::Call,
            unique_id: Uuid::new_v4().to_string(),
            action,
            payload: serde_json::to_value(payload)?,
        })
    }

    /// Extract the payload as a specific type
    pub fn payload_as<T>(&self) -> OcppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_value(self.payload.clone())?)
    }
}

/// OCPP CallResult message (successful response)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallResultMessage {
    /// Message type identifier (always 3 for CallResult)
    #[serde(rename = "0")]
    pub message_type: MessageType,
    /// Unique message identifier (same as corresponding Call)
    #[serde(rename = "1")]
    pub unique_id: String,
    /// Response payload
    #[serde(rename = "2")]
    pub payload: serde_json::Value,
}

impl CallResultMessage {
    /// Create a new CallResult message
    pub fn new<T>(unique_id: String, payload: T) -> OcppResult<Self>
    where
        T: Serialize,
    {
        Ok(CallResultMessage {
            message_type: MessageType::CallResult,
            unique_id,
            payload: serde_json::to_value(payload)?,
        })
    }

    /// Extract the payload as a specific type
    pub fn payload_as<T>(&self) -> OcppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_value(self.payload.clone())?)
    }
}

/// OCPP CallError message (error response)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallErrorMessage {
    /// Message type identifier (always 4 for CallError)
    #[serde(rename = "0")]
    pub message_type: MessageType,
    /// Unique message identifier (same as corresponding Call)
    #[serde(rename = "1")]
    pub unique_id: String,
    /// Error code
    #[serde(rename = "2")]
    pub error_code: CallErrorCode,
    /// Human-readable error description
    #[serde(rename = "3")]
    pub error_description: String,
    /// Additional error details
    #[serde(rename = "4")]
    pub error_details: serde_json::Value,
}

impl CallErrorMessage {
    /// Create a new CallError message
    pub fn new(
        unique_id: String,
        error_code: CallErrorCode,
        error_description: String,
        error_details: Option<serde_json::Value>,
    ) -> Self {
        CallErrorMessage {
            message_type: MessageType::CallError,
            unique_id,
            error_code,
            error_description,
            error_details: error_details.unwrap_or(serde_json::Value::Object(Default::default())),
        }
    }
}

/// Raw OCPP message as received over the wire (array format)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawMessage {
    /// Call message: [2, "unique_id", "action", payload]
    Call(u8, String, String, serde_json::Value),
    /// CallResult message: [3, "unique_id", payload]
    CallResult(u8, String, serde_json::Value),
    /// CallError message: [4, "unique_id", "error_code", "error_description", error_details]
    CallError(u8, String, String, String, serde_json::Value),
}

impl RawMessage {
    /// Convert raw message to typed message
    pub fn into_message(self) -> OcppResult<Message> {
        match self {
            RawMessage::Call(msg_type, unique_id, action, payload) => {
                if msg_type != 2 {
                    return Err(OcppError::InvalidMessageType(msg_type));
                }
                Ok(Message::Call(CallMessage {
                    message_type: MessageType::Call,
                    unique_id,
                    action,
                    payload,
                }))
            }
            RawMessage::CallResult(msg_type, unique_id, payload) => {
                if msg_type != 3 {
                    return Err(OcppError::InvalidMessageType(msg_type));
                }
                Ok(Message::CallResult(CallResultMessage {
                    message_type: MessageType::CallResult,
                    unique_id,
                    payload,
                }))
            }
            RawMessage::CallError(
                msg_type,
                unique_id,
                error_code_str,
                error_description,
                error_details,
            ) => {
                if msg_type != 4 {
                    return Err(OcppError::InvalidMessageType(msg_type));
                }

                let error_code = match error_code_str.as_str() {
                    "NotImplemented" => CallErrorCode::NotImplemented,
                    "NotSupported" => CallErrorCode::NotSupported,
                    "InternalError" => CallErrorCode::InternalError,
                    "ProtocolError" => CallErrorCode::ProtocolError,
                    "SecurityError" => CallErrorCode::SecurityError,
                    "FormationViolation" => CallErrorCode::FormationViolation,
                    "PropertyConstraintViolation" => CallErrorCode::PropertyConstraintViolation,
                    "OccurrenceConstraintViolation" => CallErrorCode::OccurrenceConstraintViolation,
                    "TypeConstraintViolation" => CallErrorCode::TypeConstraintViolation,
                    "GenericError" => CallErrorCode::GenericError,
                    _ => {
                        return Err(OcppError::ProtocolViolation {
                            message: format!("Unknown error code: {}", error_code_str),
                        })
                    }
                };

                Ok(Message::CallError(CallErrorMessage {
                    message_type: MessageType::CallError,
                    unique_id,
                    error_code,
                    error_description,
                    error_details,
                }))
            }
        }
    }
}

impl From<Message> for RawMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::Call(msg) => RawMessage::Call(2, msg.unique_id, msg.action, msg.payload),
            Message::CallResult(msg) => RawMessage::CallResult(3, msg.unique_id, msg.payload),
            Message::CallError(msg) => RawMessage::CallError(
                4,
                msg.unique_id,
                msg.error_code.as_str().to_string(),
                msg.error_description,
                msg.error_details,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_call_message_creation() {
        let payload = json!({"idTag": "12345"});
        let msg = CallMessage::new("Authorize".to_string(), &payload).unwrap();

        assert_eq!(msg.action, "Authorize");
        assert_eq!(msg.message_type, MessageType::Call);
        assert!(!msg.unique_id.is_empty());
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_call_result_message_creation() {
        let payload = json!({"idTagInfo": {"status": "Accepted"}});
        let msg = CallResultMessage::new("12345".to_string(), &payload).unwrap();

        assert_eq!(msg.unique_id, "12345");
        assert_eq!(msg.message_type, MessageType::CallResult);
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_call_error_message_creation() {
        let msg = CallErrorMessage::new(
            "12345".to_string(),
            CallErrorCode::NotImplemented,
            "Action not implemented".to_string(),
            None,
        );

        assert_eq!(msg.unique_id, "12345");
        assert_eq!(msg.message_type, MessageType::CallError);
        assert_eq!(msg.error_code, CallErrorCode::NotImplemented);
        assert_eq!(msg.error_description, "Action not implemented");
    }

    #[test]
    fn test_message_enum() {
        let payload = json!({"test": "data"});
        let msg = Message::call("TestAction".to_string(), &payload).unwrap();

        assert_eq!(msg.message_type(), MessageType::Call);
        assert!(!msg.unique_id().is_empty());
    }

    #[test]
    fn test_raw_message_conversion() {
        let payload = json!({"idTag": "12345"});
        let call_msg = CallMessage::new("Authorize".to_string(), &payload).unwrap();
        let message = Message::Call(call_msg.clone());

        let raw: RawMessage = message.into();
        let converted_back = raw.into_message().unwrap();

        if let Message::Call(converted_call) = converted_back {
            assert_eq!(converted_call.action, call_msg.action);
            assert_eq!(converted_call.unique_id, call_msg.unique_id);
            assert_eq!(converted_call.payload, call_msg.payload);
        } else {
            panic!("Expected Call message");
        }
    }

    #[test]
    fn test_call_message_serialization() {
        let payload = json!({"idTag": "TEST123"});
        let msg = CallMessage::new("Authorize".to_string(), &payload).unwrap();

        let json_str = serde_json::to_string(&msg).unwrap();
        let deserialized: CallMessage = serde_json::from_str(&json_str).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_call_error_serialization() {
        let msg = CallErrorMessage::new(
            "test-id".to_string(),
            CallErrorCode::InternalError,
            "Test error".to_string(),
            Some(json!({"detail": "more info"})),
        );

        let json_str = serde_json::to_string(&msg).unwrap();
        let deserialized: CallErrorMessage = serde_json::from_str(&json_str).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_raw_message_invalid_type() {
        let raw = RawMessage::Call(5, "test".to_string(), "Action".to_string(), json!({}));
        let result = raw.into_message();
        assert!(result.is_err());

        if let Err(OcppError::InvalidMessageType(5)) = result {
            // Expected
        } else {
            panic!("Expected InvalidMessageType error");
        }
    }

    #[test]
    fn test_payload_extraction() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestPayload {
            id_tag: String,
        }

        let payload = TestPayload {
            id_tag: "TEST123".to_string(),
        };

        let msg = CallMessage::new("Authorize".to_string(), &payload).unwrap();
        let extracted: TestPayload = msg.payload_as().unwrap();

        assert_eq!(extracted, payload);
    }
}

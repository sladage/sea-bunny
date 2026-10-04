use std::collections::HashMap;

use serde::{Serialize, Serializer};

use super::BreakoutRoomMode;

/// Assignment of attendee ids to breakout room numbers (0-based).
pub type AttendeeMap = HashMap<i64, u32>;

/// Body for configuring breakout rooms of a conversation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureBreakoutRooms {
    pub mode: BreakoutRoomMode,
    /// Number of rooms.
    pub amount: u32,
    /// Only used in [`BreakoutRoomMode::Manual`].
    #[serde(serialize_with = "attendee_map_as_json")]
    pub attendee_map: AttendeeMap,
}

/// The API expects the attendee map as a JSON-encoded string.
pub(crate) fn attendee_map_as_json<S: Serializer>(
    map: &AttendeeMap,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let encoded = serde_json::to_string(map).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attendee_map_is_encoded_as_string() {
        let body = ConfigureBreakoutRooms {
            mode: BreakoutRoomMode::Manual,
            amount: 2,
            attendee_map: HashMap::from([(7, 1)]),
        };
        let json = serde_json::to_value(body).unwrap();
        assert_eq!(json["attendeeMap"], r#"{"7":1}"#);
        assert_eq!(json["mode"], 2);
    }
}

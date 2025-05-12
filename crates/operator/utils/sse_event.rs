use axum::response::sse::Event;

// Define our trait
pub trait EventParser {
    fn parse_from_string(value: String) -> Result<Event, std::io::Error>;
}

// Implement the trait for our newtype
impl EventParser for Event {
    fn parse_from_string(value: String) -> Result<Event, std::io::Error> {
        let mut event = Event::default();
        for line in value.lines() {
            if line.is_empty() {
                continue;
            } else if line.starts_with("data:") {
                let data = line.trim_start_matches("data:").trim();
                event = event.data(data);
            } else if line.starts_with("event:") {
                let event_type = line.trim_start_matches("event:").trim();
                event = event.event(event_type);
            } else if line.starts_with("id:") {
                let id = line.trim_start_matches("id:").trim();
                event = event.id(id);
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid SSE line: {line}"),
                ));
            }
        }
        Ok(event)
    }
}

// Create a newtype wrapper
pub struct EventExt(pub Event);

// Implement TryFrom for our newtype
impl TryFrom<String> for EventExt {
    type Error = std::io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(Event::parse_from_string(value)?))
    }
}

// Implement into from EventExt to Event
impl From<EventExt> for Event {
    fn from(event_ext: EventExt) -> Self {
        event_ext.0
    }
}

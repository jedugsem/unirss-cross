use crate::Message;

pub fn back_message(tab: u8) -> Message {
    match tab {
        _ => Message::Nothing,
    }
}

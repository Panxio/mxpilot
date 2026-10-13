pub mod action;
pub mod device;
pub mod error;

pub use action::Action;
pub use device::VirtualInput;
pub use error::UinputError;
pub use evdev::KeyCode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_constructors() {
        assert_eq!(Action::super_key(), Action::SingleKey(KeyCode::KEY_LEFTMETA));

        if let Action::KeyCombo { modifiers, key } = Action::copy() {
            assert_eq!(modifiers, vec![KeyCode::KEY_LEFTCTRL]);
            assert_eq!(key, KeyCode::KEY_C);
        } else {
            panic!("Expected KeyCombo for copy");
        }
    }
}

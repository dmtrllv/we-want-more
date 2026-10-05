use evdev::{AttributeSet, EventType, InputEvent, KeyCode, RelativeAxisCode, uinput::VirtualDevice};

use crate::{display_manager::Position, platform::Mouse};

#[derive(Debug)]
pub struct LinuxMouse {
    device: VirtualDevice,
    position: Position
}

impl Mouse for LinuxMouse {
    fn new(init_position: Position) -> Result<Self, String> {
        let mut rel_axes = AttributeSet::<RelativeAxisCode>::new();
        rel_axes.insert(RelativeAxisCode::REL_X);
        rel_axes.insert(RelativeAxisCode::REL_Y);

        let mut keys = AttributeSet::<KeyCode>::new();
        keys.insert(KeyCode::BTN_LEFT);
        keys.insert(KeyCode::BTN_RIGHT);
        keys.insert(KeyCode::BTN_MIDDLE);

        let device = VirtualDevice::builder()
            .map_err(|e| e.to_string())?
            .name("cross-platform-input-sync")
            .with_relative_axes(&rel_axes)
            .map_err(|e| e.to_string())?
            .with_keys(&keys)
            .map_err(|e| e.to_string())?
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self { position: init_position, device })
    }

    fn move_absolute(&mut self, position: Position) {
        let dx = position.0 - self.position.0;
        let dy = position.1 - self.position.1;
        println!("move mouse {:?} -> dx {} dy {}", position, dx, dy);
        self.position = position;
        let _ = self.device.emit(&[
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_X.0, dx as i32),
            InputEvent::new(EventType::RELATIVE.0, RelativeAxisCode::REL_Y.0, dy as i32),
            InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0),
        ]).inspect_err(|e| {
            println!("Mouse error: {e:?}");
        });
    }
}

use crate::devices::{graphics_tablet::GraphicsTabletDevice, touchpad::TouchpadDevice};

pub(crate) enum DriverState {
    Initialized {
        tablet: GraphicsTabletDevice,
        touchpad: TouchpadDevice,
    },
    Uninitialized,
}

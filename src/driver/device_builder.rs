use std::io;

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisCode, AttributeSet, BusType, InputId, KeyCode, PropType, UinputAbsSetup
};

pub(crate) fn graphic_tablet(width: i32, height: i32) -> io::Result<VirtualDevice> {
    println!("device setup. width {} height {}", width, height);
    let mut touchetab_keys: AttributeSet<KeyCode> = AttributeSet::new();
    touchetab_keys.insert(KeyCode::BTN_STYLUS);
    touchetab_keys.insert(KeyCode::BTN_TOOL_PEN);
    touchetab_keys.insert(KeyCode::BTN_TOUCH);

    let mut touchetab_props: AttributeSet<PropType> = AttributeSet::new();
    touchetab_props.insert(PropType::DIRECT);
    touchetab_props.insert(PropType::POINTER);

    evdev::uinput::VirtualDevice::builder()?
        .name("touchetab")
        .with_properties(&touchetab_props)?
        .with_keys(&touchetab_keys)?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_X,
            AbsInfo::new(0, 0, width, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_Y,
            AbsInfo::new(0, 0, height, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_PRESSURE,
            AbsInfo::new(0, 0, 4096, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_DISTANCE,
            AbsInfo::new(0, 0, 1024, 0, 0, 100),
        ))?
        .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0001, 0x1))
        .build()
}

pub(crate) fn touchpad(width: i32, height: i32) -> io::Result<VirtualDevice> {
    let mut touchepad_keys: AttributeSet<KeyCode> = AttributeSet::new();
    touchepad_keys.insert(KeyCode::BTN_TOUCH);
    touchepad_keys.insert(KeyCode::BTN_TOOL_FINGER);
    touchepad_keys.insert(KeyCode::BTN_TOOL_DOUBLETAP);
    touchepad_keys.insert(KeyCode::BTN_TOOL_TRIPLETAP);

    let mut touchepad_props: AttributeSet<PropType> = AttributeSet::new();
    touchepad_props.insert(PropType::POINTER);

    evdev::uinput::VirtualDevice::builder()?
        .name("touchepad")
        .with_properties(&touchepad_props)?
        .with_keys(&touchepad_keys)?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_MT_SLOT,
            AbsInfo::new(0, 0, 10, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_MT_TRACKING_ID,
            AbsInfo::new(0, 0, 65535, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_MT_POSITION_X,
            AbsInfo::new(0, 0, width, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_MT_POSITION_Y,
            AbsInfo::new(0, 0, height, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_X,
            AbsInfo::new(0, 0, width, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisCode::ABS_Y,
            AbsInfo::new(0, 0, height, 0, 0, 100),
        ))?
        .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0002, 0x1))
        .build()
}

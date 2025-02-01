use std::io;

use evdev::{
    uinput::VirtualDevice, AbsInfo, AbsoluteAxisType, AttributeSet, BusType, InputId, Key,
    PropType, UinputAbsSetup,
};

pub(crate) fn graphic_tablet(width: i32, height: i32) -> io::Result<VirtualDevice> {
    println!("device setup. width {} height {}", width, height);
    let mut touchetab_keys: AttributeSet<Key> = AttributeSet::new();
    touchetab_keys.insert(Key::BTN_STYLUS);
    touchetab_keys.insert(Key::BTN_TOOL_PEN);
    touchetab_keys.insert(Key::BTN_TOUCH);

    let mut touchetab_props: AttributeSet<PropType> = AttributeSet::new();
    touchetab_props.insert(PropType::DIRECT);
    touchetab_props.insert(PropType::POINTER);

    evdev::uinput::VirtualDeviceBuilder::new()?
        .name("touchetab")
        .with_properties(&touchetab_props)?
        .with_keys(&touchetab_keys)?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_X,
            AbsInfo::new(0, 0, width, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_Y,
            AbsInfo::new(0, 0, height, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_PRESSURE,
            AbsInfo::new(0, 0, 4096, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_DISTANCE,
            AbsInfo::new(0, 0, 1024, 0, 0, 100),
        ))?
        .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0001, 0x1))
        .build()
}

pub(crate) fn touchpad(width: i32, height: i32) -> io::Result<VirtualDevice> {
    let mut touchepad_keys: AttributeSet<Key> = AttributeSet::new();
    touchepad_keys.insert(Key::BTN_TOUCH);
    touchepad_keys.insert(Key::BTN_TOOL_FINGER);
    touchepad_keys.insert(Key::BTN_TOOL_DOUBLETAP);

    let mut touchepad_props: AttributeSet<PropType> = AttributeSet::new();
    touchepad_props.insert(PropType::POINTER);

    evdev::uinput::VirtualDeviceBuilder::new()?
        .name("touchepad")
        .with_properties(&touchepad_props)?
        .with_keys(&touchepad_keys)?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_SLOT,
            AbsInfo::new(0, 0, 10, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_TRACKING_ID,
            AbsInfo::new(0, 0, i32::MAX, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_POSITION_X,
            AbsInfo::new(0, 0, height, 0, 0, 100),
        ))?
        .with_absolute_axis(&UinputAbsSetup::new(
            AbsoluteAxisType::ABS_MT_POSITION_Y,
            AbsInfo::new(0, 0, width, 0, 0, 100),
        ))?
        .input_id(InputId::new(BusType::BUS_USB, 0x5120, 0x0002, 0x1))
        .build()
}

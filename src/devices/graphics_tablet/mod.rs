use std::cmp::PartialEq;
use std::error::Error;
use std::io;

use crate::data::events::ToucheEvent;

#[cfg(target_os = "linux")]
use evdev::{
    AbsInfo, AbsoluteAxisCode, AbsoluteAxisEvent, AttributeSet, BusType, InputEvent, InputId,
    KeyCode, KeyEvent, PropType, UinputAbsSetup, uinput::VirtualDevice,
};
use log::{error, info, trace};

#[cfg(target_os = "linux")]
pub(crate) struct GraphicsTabletDevice {
    device: VirtualDevice,
}

#[cfg(target_os = "linux")]
impl GraphicsTabletDevice {
    pub(crate) fn new(width: i32, height: i32) -> io::Result<GraphicsTabletDevice> {
        println!("device setup. width {} height {}", width, height);
        let mut touche_tablet_keys: AttributeSet<KeyCode> = AttributeSet::new();

        // defining stylus capabilities
        touche_tablet_keys.insert(KeyCode::BTN_STYLUS);
        touche_tablet_keys.insert(KeyCode::BTN_TOOL_PEN);
        touche_tablet_keys.insert(KeyCode::BTN_TOUCH);

        // defining on-tablet buttons
        touche_tablet_keys.insert(KeyCode::BTN_0);
        touche_tablet_keys.insert(KeyCode::BTN_1);
        touche_tablet_keys.insert(KeyCode::BTN_2);
        touche_tablet_keys.insert(KeyCode::BTN_3);
        touche_tablet_keys.insert(KeyCode::BTN_4);
        touche_tablet_keys.insert(KeyCode::BTN_5);
        touche_tablet_keys.insert(KeyCode::BTN_6);
        touche_tablet_keys.insert(KeyCode::BTN_7);
        touche_tablet_keys.insert(KeyCode::BTN_8);
        touche_tablet_keys.insert(KeyCode::BTN_9);

        // props set accordingly to specifications in Linux kernel docs
        let mut touche_tablet_props: AttributeSet<PropType> = AttributeSet::new();
        touche_tablet_props.insert(PropType::DIRECT);
        touche_tablet_props.insert(PropType::POINTER);

        let device = evdev::uinput::VirtualDevice::builder()?
            .name("touchetab")
            .with_properties(&touche_tablet_props)?
            .with_keys(&touche_tablet_keys)?
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
            .build()?;
        Ok(GraphicsTabletDevice { device })
    }

    pub(crate) fn emit(&mut self, touche_data: &[ToucheEvent]) -> Result<(), io::Error> {
        let mut tablet_events: Vec<InputEvent> = vec![];
        for event in touche_data {
            match event {
                ToucheEvent::Touch { .. } => {
                    // touch frame - ignore
                }
                ToucheEvent::Stylus {
                    x,
                    y,
                    pressed,
                    pressure,
                } => {
                    let tool_pen_event = *KeyEvent::new(KeyCode::BTN_TOOL_PEN, 1);
                    let x_event = *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_X, *x);
                    let y_event = *AbsoluteAxisEvent::new(AbsoluteAxisCode::ABS_Y, *y);
                    let touch_event = *KeyEvent::new(KeyCode::BTN_TOUCH, (*pressed).into());

                    tablet_events.push(tool_pen_event);
                    tablet_events.push(x_event);
                    tablet_events.push(y_event);
                    tablet_events.push(touch_event);

                    let pressure_int = (pressure * 4096.0) as i32; // Assuming max pressure is 4096
                    tablet_events.push(*AbsoluteAxisEvent::new(
                        AbsoluteAxisCode::ABS_PRESSURE,
                        pressure_int,
                    ));
                }
                ToucheEvent::Button { button_id, pressed } => {
                    let key_code = match button_id {
                        0 => KeyCode::BTN_0,
                        1 => KeyCode::BTN_1,
                        2 => KeyCode::BTN_2,
                        3 => KeyCode::BTN_3,
                        4 => KeyCode::BTN_4,
                        5 => KeyCode::BTN_5,
                        6 => KeyCode::BTN_6,
                        7 => KeyCode::BTN_7,
                        8 => KeyCode::BTN_8,
                        9 => KeyCode::BTN_9,
                        _ => {
                            error!("unsupported button id {}", button_id);
                            continue;
                        }
                    };
                    tablet_events.push(*KeyEvent::new(key_code, (*pressed).into()));
                }
            }
        }
        if !tablet_events.is_empty() {
            return self.device.emit(&tablet_events);
        }
        Result::Ok(())
    }
}

#[cfg(target_os = "windows")]
use windows::UI::Input::Preview::Injection::{InjectedInputPenInfo, InjectedInputPenParameters, InjectedInputPointerInfo, InjectedInputPointerOptions, InjectedInputVisualizationMode, InputInjector};

#[cfg(target_os = "windows")]
pub(crate) struct GraphicsTabletDevice {
    input_injector: InputInjector,
    width: i32,
    height: i32,
    pointer_id: u32, // TODO: get one from Android
    pointer_state: PointerState
}

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
struct PointerState {
    new_pointer_notified: bool,
    in_range: bool,
    in_contact: bool,
    first_button: bool,
    second_button: bool,
    primary: bool,
    confidence: bool,
    pointer_down: bool,
    pointer_up: bool,
    capture_changed: bool,
}

#[cfg(windows)]
impl PartialEq for PointerState {
    fn eq(&self, other: &Self) -> bool {
        self.new_pointer_notified == other.new_pointer_notified &&
            self.in_range == other.in_range &&
            self.in_contact == other.in_contact &&
            self.first_button == other.first_button &&
            self.second_button == other.second_button &&
            self.primary == other.primary &&
            self.confidence == other.confidence &&
            self.pointer_down == other.pointer_down &&
            self.pointer_up == other.pointer_up &&
            self.capture_changed == other.capture_changed
    }
}

#[cfg(windows)]
impl PointerState {
    fn new() -> PointerState {
        PointerState {
            new_pointer_notified: false,
            in_range: true, // it is always in range
            in_contact: false,
            first_button: false,
            second_button: false,
            primary: false,
            confidence: false,
            pointer_down: false,
            pointer_up: false,
            capture_changed: false,
        }
    }

    fn calculate_pointer_options(&self, previous_state: PointerState) -> InjectedInputPointerOptions {
        info!("windows: previous state: {:?}, current state: {:?}", previous_state, self);
            let mut options = InjectedInputPointerOptions::None;

            if previous_state == *self { options |= InjectedInputPointerOptions::Update; }

            // notifying about the new pointer popping up...
            if !self.new_pointer_notified { options |= InjectedInputPointerOptions::New; }
            if self.in_range { options |= InjectedInputPointerOptions::InRange; }
            if self.in_contact { options |= InjectedInputPointerOptions::InContact; }
            if self.first_button { options |= InjectedInputPointerOptions::FirstButton; }
            if self.second_button { options |= InjectedInputPointerOptions::SecondButton; }
            if self.primary { options |= InjectedInputPointerOptions::Primary; }
            if self.confidence { options |= InjectedInputPointerOptions::Confidence; }

            // if the pointer is down/up, and it was not down/up before, notifying about it
            if self.pointer_down {
                options |= InjectedInputPointerOptions::PointerDown;
            }
            if self.pointer_up {
                options |= InjectedInputPointerOptions::PointerUp;
            }

            if self.capture_changed { options |= InjectedInputPointerOptions::CaptureChanged; }

            options
    }
}

#[cfg(target_os = "windows")]
impl GraphicsTabletDevice {
    pub(crate) fn new(width: i32, height: i32) -> Result<GraphicsTabletDevice, Box<dyn Error>> {
        let injector = InputInjector::TryCreate().expect("failed to create input injector");
        injector.InitializePenInjection(InjectedInputVisualizationMode::Default)?;

        let pointer_state = PointerState::new();
        Ok(GraphicsTabletDevice { input_injector: injector, width, height, pointer_state, pointer_id: 420 })
    }

    pub(crate) fn emit(&mut self, touche_data: &[ToucheEvent]) -> Result<(), io::Error> {
        let injector_data_vec: Vec<InjectedInputPenInfo> = touche_data.iter().filter_map(|event| {
            info!("windows: event: {:?}", event);
            match event {
                ToucheEvent::Stylus { x, y, pressed, pressure } => {
                    let previous_state = self.pointer_state.clone();

                    self.pointer_state.in_range = true;
                    self.pointer_state.in_contact = *pressed;
                    self.pointer_state.first_button = *pressed;
                    self.pointer_state.primary = true;
                    self.pointer_state.confidence = true;
                    self.pointer_state.pointer_down = *pressed;
                    self.pointer_state.pointer_up = !*pressed;
                    let pointer_options = self.pointer_state.calculate_pointer_options(previous_state);
                    self.pointer_state.new_pointer_notified = true;

                    let mut pointer_info = InjectedInputPointerInfo::default();

                    pointer_info.PixelLocation.PositionX = *x;
                    pointer_info.PixelLocation.PositionY = *y;
                    pointer_info.PointerOptions = pointer_options;

                    let pen_info = InjectedInputPenInfo::new().expect("windows: pen info: failed to create pen info");
                    pen_info.SetPressure(*pressure as f64).expect("windows: pen info: failed to set pressure");
                    pen_info.SetPenParameters(InjectedInputPenParameters::Pressure).expect("windows: pen info: failed to set pen parameters");
                    pen_info.SetPointerInfo(pointer_info).expect("windows: pen info: failed to set pointer info");
                    Some(pen_info)
                }
                ToucheEvent::Touch { .. } => { None }
                ToucheEvent::Button { .. } => { None /* not supported on Windows */ }
            }
        }).collect();

        for injector_data in injector_data_vec {
            let result = self.input_injector.InjectPenInput(Some(&injector_data));
            info!("windows: inject result: {result:?}");
        }

        Ok(())
    }
}

pub(crate) mod events;
pub mod errors;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "ToucheInput.Action")]
pub(crate) enum Action {
    #[serde(rename = "ToucheInput.Action.Init")]
    Init
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum ToucheData {
    #[serde(rename = "ToucheInput.Screen")]
    ScreenSize {
        x: i32,
        y: i32,
    },
    #[serde(rename = "ToucheInput.Stylus")]
    StylusFrame {
        x: i32,
        y: i32,
        pressed: bool,
        pressure: f32,
    },
    #[serde(rename = "ToucheInput.Finger")]
    TouchFrame {
        x: i32,
        y: i32,
        #[serde(rename = "touchId")] touch_id: i32,
        pressed: bool,
    },
    #[serde(rename = "ToucheInput.Button")]
    ButtonFrame {
        button_id: i32,
        pressed: bool,
    },
    #[serde(rename = "ToucheInput.Action")]
    Action(Action)
}


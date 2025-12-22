#[derive(Debug)]
pub enum ToucheEvent {
    Stylus {
        x: i32,
        y: i32,
        pressed: bool,
        pressure: f32,
    },
    Touch {
        x: i32,
        y: i32,
        touch_id: i32,
        pressed: bool,
    },
    Button {
        button_id: i32,
        pressed: bool,
    },
}

// TODO: remove enum umbrella

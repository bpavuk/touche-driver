use std::string::FromUtf8Error;

use log::trace;

pub(crate) enum ToucheData {
    ScreenSize {
        x: i32,
        y: i32,
    },
    StylusFrame {
        x: i32,
        y: i32,
        pressed: bool,
        pressure: Option<f32>,
    },
    TouchFrame {
        x: i32,
        y: i32,
        touch_id: i32,
        pressed: bool,
    },
    ButtonFrame {
        button_id: i32,
        pressed: bool,
    },
}

pub(crate) fn parse_touche_data(input: &Vec<u8>) -> Result<Vec<ToucheData>, FromUtf8Error> {
    let info_string = String::from_utf8(input.to_owned())?;
    trace!("touche input data:\n{}", info_string);

    let token_table: Vec<Vec<&str>> = info_string
        .split("\n")
        .map(|it| it.split("\t").collect::<Vec<&str>>())
        .collect();

    let mut data: Vec<ToucheData> = vec![];
    for token_row in token_table {
        match token_row[0] {
            "X" => {
                let x = token_row[1].parse::<i32>();
                if x.is_err() {
                    continue;
                }
                let x = x.unwrap();

                let y = token_row[2].parse::<i32>();
                if y.is_err() {
                    continue;
                }
                let y = y.unwrap();

                data.push(ToucheData::ScreenSize { x, y });
            }
            "S" => {
                if let Some([x, y, pressed]) = token_row.get(1..=3) {
                    let x = match x.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    let y = match y.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };

                    let pressed = match pressed.parse::<i32>() {
                        Ok(x) => x,
                        Err(_) => continue,
                    };
                    let pressed = pressed == 1;

                    let pressure = if pressed {
                        match token_row.get(4) {
                            Some(pressure) => match pressure.parse::<f32>() {
                                Ok(pressure) => Some(pressure),
                                Err(_) => {
                                    continue;
                                }
                            },
                            None => None,
                        }
                    } else {
                        None
                    };
                    data.push(ToucheData::StylusFrame {
                        x: x as i32,
                        y: y as i32,
                        pressed,
                        pressure,
                    });
                };
            }
            "F" => {
                if let Some([x, y, pressed, touch_id]) = token_row.get(1..=4) {
                    let x = x.parse::<f32>();
                    if x.is_err() {
                        trace!("error parsing F.x");
                        continue;
                    }
                    let x = x.unwrap();

                    let y = y.parse::<f32>();
                    if y.is_err() {
                        trace!("error parsing F.y");
                        continue;
                    }
                    let y = y.unwrap();

                    let pressed = pressed.parse::<i32>();
                    if pressed.is_err() {
                        trace!("error parsing F.pressed");
                        continue;
                    }
                    let pressed = pressed.unwrap() == 1;

                    let touch_id = touch_id.parse();
                    if touch_id.is_err() {
                        trace!("error parsing F.touch_id");
                        continue;
                    }
                    let touch_id = touch_id.unwrap();

                    data.push(ToucheData::TouchFrame {
                        x: x as i32,
                        y: y as i32,
                        touch_id,
                        pressed,
                    });
                };
            }
            "B" => {
                if let Some([button_id, pressed]) = token_row.get(1..=2) {
                    let button_id = button_id.parse::<i32>();
                    if button_id.is_err() {
                        trace!("error parsing B.button_id");
                        continue;
                    }
                    let button_id = button_id.unwrap();

                    let pressed = pressed.parse::<i32>();
                    if pressed.is_err() {
                        trace!("error parsing B.pressed");
                        continue;
                    }
                    let pressed = pressed.unwrap() == 1;

                    data.push(ToucheData::ButtonFrame { button_id, pressed });
                };
            }
            _ => {}
        }
    }

    Result::Ok(data)
}

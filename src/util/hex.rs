impl Hex for u16 {
    fn to_hex_str(&self) -> String {
        let mut string = String::new();
        let mut number = *self;

        loop {
            let modulo = number % 16;
            let character = char::from_digit(modulo.into(), 16).unwrap();
            string.insert(0, character);
            number /= 16;
            if number / 16 == 0 {
                string.insert(0, char::from_digit(number.into(), 16).unwrap());
                break;
            }
        }

        string
    }
}

#[allow(dead_code)]
/// sometimes useful in debugging
trait Hex {
    fn to_hex_str(&self) -> String;
}

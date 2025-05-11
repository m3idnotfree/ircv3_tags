#[allow(unused_variables)]
pub trait CharValidator {
    fn is_valid_char(&self, c: char) -> bool {
        true
    }
    fn is_valid_start_char(&self, c: char) -> bool {
        true
    }
    fn is_invalid_char(&self, s: &str) -> bool {
        false
    }
    fn while_valid<'a>(&self, input: &'a str, first_char: char) -> (&'a str, &'a str) {
        let mut position = 0;
        let mut chars = input.chars();

        chars.next();
        position += first_char.len_utf8();

        for c in chars {
            if !self.is_valid_char(c) {
                break;
            }
            position += c.len_utf8();
        }

        (&input[position..], &input[..position])
    }
}

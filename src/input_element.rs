pub struct InputTextElement {
    text: String,
    cursor_position: i64,
}

impl InputTextElement {
    pub fn new(text: String) -> Self {
        Self {
            cursor_position: text.len() as i64 - 1,
            text,
        }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn cursor_position(&self) -> i64 {
        self.cursor_position
    }

    pub fn add_char(&mut self, ch: char) {
        self.cursor_position += 1;
        self.text.insert(self.cursor_position as usize, ch);
    }

    pub fn remove_char(&mut self) {
        if self.cursor_position == -1 {
            return;
        }

        self.text.remove(self.cursor_position as usize);
        self.cursor_position -= 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position == self.text.len() as i64 - 1 {
            return;
        }

        self.text.remove(self.cursor_position as usize + 1);
    }

    pub fn cursor_left(&mut self) {
        self.cursor_position = std::cmp::max(self.cursor_position - 1, -1);
    }

    pub fn cursor_right(&mut self) {
        self.cursor_position = std::cmp::min(self.cursor_position + 1, self.text.len() as i64 - 1);
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = -1;
    }
}

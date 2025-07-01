pub struct ParsePosition {
    index: isize,
    error_index: isize,
}

impl ParsePosition {
    pub fn new(index: isize) -> Self {
        ParsePosition {
            index,
            error_index: -1,
        }
    }

    pub fn get_index(&self) -> isize {
        self.index
    }

    pub fn set_index(&mut self, index: isize) {
        self.index = index;
    }

    pub fn get_error_index(&self) -> isize {
        self.error_index
    }

    pub fn set_error_index(&mut self, error_index: isize) {
        self.error_index = error_index;
    }
}

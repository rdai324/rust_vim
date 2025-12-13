
use regex::Regex;
use ropey::Rope;
use std::cmp;
use std::fs::File;
use std::io::BufReader;

#[derive(PartialEq, Eq, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    Visual,
}

#[derive(Debug)]
pub struct EditorModel {
    pub rope: Rope,
    pub cursor_x: usize, // column in chars
    pub cursor_y: usize, // line index
    pub mode: Mode,
    pub file_name: String,
    pub command: String,
    pub search_query: String,

    pub visual_start: Option<(usize, usize)>,

    pub status: String,
    pub search_matches: Vec<(usize, usize)>, // (line, col)
}

impl EditorModel {
    pub fn new(file_name: &str) -> Self {
        Self {
            rope: Rope::from_reader(BufReader::new(File::open(&file_name).unwrap())).unwrap(),
            cursor_x: 0,
            cursor_y: 0,
            mode: Mode::Normal,
            file_name: String::from(file_name),
            command: String::new(),
            search_query: String::new(),
            visual_start: None,
            status: "-- NORMAL --".into(),
            search_matches: vec![],
        }
    }

    // Helpers to convert between (line, col) and char index
    pub fn line_start_char(&self, line: usize) -> usize {
        // line_to_char returns starting char index of the line
        self.rope.line_to_char(line)
    }

    pub fn line_len_chars(&self, line: usize) -> usize {
        let start = self.rope.line_to_char(line);
        let next = if line + 1 < self.rope.len_lines() {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        next - start
    }

    pub fn insert_char(&mut self, ch: char, char_idx: usize) {
        self.rope.insert_char(char_idx, ch);
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let char_idx = self.line_start_char(self.cursor_y) + self.cursor_x;
            self.rope.remove((char_idx - 1)..char_idx);
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            // join with previous line
            let prev_len = self.line_len_chars(self.cursor_y - 1);
            let char_idx = self.line_start_char(self.cursor_y);
            // remove the newline char between lines (i.e. do nothing special; inserting/removing will handle)
            // Actually rope represents lines without an explicit newline at the end, but line boundaries handled by indices
            // To join, remove the line break by removing range char_idx..char_idx (no-op) but we need to move cursor
            self.cursor_y -= 1;
            self.cursor_x = prev_len;
        }
    }

    pub fn newline(&mut self) {
        let char_idx = self.line_start_char(self.cursor_y) + self.cursor_x;
        self.rope.insert(char_idx, "
");
        self.cursor_y += 1;
        self.cursor_x = 0;
    }

    pub fn delete_char(&mut self, char_idx: usize) {
        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
        }
    }

    // Movement
    pub fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.line_len_chars(self.cursor_y);
        }
    }

    pub fn move_right(&mut self) {
        let len = self.line_len_chars(self.cursor_y);
        if self.cursor_x < len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.rope.len_lines() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = cmp::min(self.cursor_x, self.line_len_chars(self.cursor_y));
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_y + 1 < self.rope.len_lines() {
            self.cursor_y += 1;
            self.cursor_x = cmp::min(self.cursor_x, self.line_len_chars(self.cursor_y));
        }
    }

    pub fn run_search(&mut self) {
        self.search_matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        let re = Regex::new(&self.search_query).unwrap();
        let lines = self.rope.len_lines();
        for y in 0..lines {
            let line = self.rope.line(y).to_string();
            for mat in re.find_iter(&line) {
                self.search_matches.push((y, mat.start()));
            }
        }
    }

    pub fn jump_next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        for (y, x) in &self.search_matches {
            if *y > self.cursor_y || (*y == self.cursor_y && *x > self.cursor_x) {
                self.cursor_y = *y;
                self.cursor_x = *x;
                return;
            }
        }
        let (y, x) = self.search_matches[0];
        self.cursor_y = y;
        self.cursor_x = x;
    }
}
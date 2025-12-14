
use regex::Regex;
use ropey::Rope;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::fs;

#[derive(Debug)]
pub struct EditorModel {
    pub rope: Rope,
    pub file_name: String,
}

impl EditorModel {
    pub fn new(file_name: &str) -> Self {
        Self {
            rope: Rope::from_reader(BufReader::new(File::open(&file_name).unwrap())).unwrap(),
            file_name: String::from(file_name),
        }
    }

    pub fn insert_char(&mut self, ch: char, char_idx: usize) {
        self.rope.insert_char(char_idx, ch);
    }

    pub fn delete_char(&mut self, char_idx: usize) {
        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
        }
    }

    pub fn run_search(&mut self, search_query: &str) -> usize {
        let mut num_matches = 0;
        if search_query.is_empty() {
            return 0;
        }
        let re = Regex::new(&search_query).unwrap();
        let lines = self.rope.len_lines();
        for y in 0..lines {
            let line = self.rope.line(y).to_string();
            for _ in re.find_iter(&line) {
                num_matches += 1;
            }
        }
        return num_matches;
    }

    pub fn save(&self) -> io::Result<()> {
        fs::write(&self.file_name, self.rope.to_string())
    }
}
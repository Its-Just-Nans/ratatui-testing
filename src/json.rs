use std::{
    collections::VecDeque,
    io::{self, Write},
};

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use serde::Serialize;
use serde_json::{
    ser::{Formatter, PrettyFormatter},
    Number, Value,
};

use crate::app::JsonEdition;

#[derive(Default)]
pub struct JsonContainer<'a> {
    pub inner: serde_json::Value,
    pub lines: Vec<Line<'a>>,
    pub save_current_pos: VecDeque<JsonEdition>,
}

impl<'a> JsonContainer<'a> {
    pub fn new(json: serde_json::Value) -> Self {
        Self {
            inner: json,
            lines: Vec::new(),
            save_current_pos: VecDeque::new(),
        }
    }

    pub fn check_same_current_pos(&self, current_pos: &VecDeque<JsonEdition>) -> bool {
        self.save_current_pos == *current_pos
    }

    pub fn create_lines(&mut self, current_pos: VecDeque<JsonEdition>) {
        if self.check_same_current_pos(&current_pos) {
            // no need to re-render
            return;
        }
        self.save_current_pos.clone_from(&current_pos);
        let mut writer = MyWriter::new(current_pos.clone());
        let formatter = PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
        self.inner.serialize(&mut ser).unwrap();
        writer.write_all(&[b'\n']).unwrap();
        self.lines = writer.inner
    }

    pub fn render(&self) -> Vec<Line<'a>> {
        self.lines.clone()
    }
}

#[derive(Default)]
pub struct MyWriter<'a> {
    pub inner: Vec<Line<'a>>,
    pub text: Vec<String>,
    internal_buf: Vec<u8>,
    current_pos: VecDeque<JsonEdition>,
    is_in_object: bool,
    is_in_array: bool,
    array_index: Option<usize>,
}

impl<'a> MyWriter<'a> {
    fn new(current_pos: VecDeque<JsonEdition>) -> Self {
        Self {
            current_pos,
            ..Default::default()
        }
    }
}

impl MyWriter<'_> {
    pub fn is_object(&mut self, part: &str) {
        if part.chars().nth(0).unwrap() == '{' {
            self.is_in_array = false;
            self.is_in_object = true;
            return;
        }
        if part.chars().nth(0).unwrap() == '[' {
            self.is_in_array = true;
            self.is_in_object = false;
            return;
        }
        if part.trim().starts_with('}') {
            self.is_in_array = false;
            self.is_in_object = false;
        }
        if part.trim().starts_with(']') {
            self.is_in_array = false;
            self.is_in_object = false;
        }
    }

    pub fn check_key(&mut self, part: &str) -> bool {
        println!("{:?}", part);
        println!("{:?}", self.current_pos.front());
        let is_match = match self.current_pos.front() {
            Some(JsonEdition::Key(front)) => {
                part.trim().starts_with(format!("\"{}\":", front).as_str())
            }
            _ => false,
        };
        if is_match {
            return self.current_pos.pop_front().is_some();
        }
        is_match
    }

    pub fn check_array(&mut self, part: &str) -> bool {
        let is_match = match self.current_pos.front() {
            Some(JsonEdition::Index(idx)) => self.array_index.unwrap() == *idx,
            _ => false,
        };
        if is_match {
            return self.current_pos.pop_front().is_some();
        }
        is_match
    }

    pub fn should_color(&mut self) -> bool {
        let part = self.text.last().unwrap().to_string();
        self.is_object(&part);
        let mut res = false;
        if self.current_pos.is_empty() {
            res = false;
        }
        if self.is_in_object {
            res = self.check_key(&part);
        }
        if self.is_in_array {
            self.array_index = self.current_pos.pop_front().and_then(|x| match x {
                JsonEdition::Index(idx) => Some(idx),
                _ => None,
            });
            res = self.check_array(&part);
        }
        if part.contains(": {\n") {
            self.is_in_object = true;
            self.is_in_array = false;
        }
        if part.contains(": [\n") {
            self.is_in_array = true;
            self.is_in_object = false;
        }
        res
    }
}

impl<'a> io::Write for MyWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.internal_buf.extend_from_slice(buf);
        let newline = self.internal_buf.iter().position(|&r| r == b'\n');
        if let Some(newline_idx) = newline {
            let curr_text = String::from_utf8_lossy(&self.internal_buf[..newline_idx + 1]);
            self.text.push(curr_text.to_string());
            let should_color = self.should_color();
            let text_to_add = self.text.last().unwrap();
            println!("{:?}", text_to_add);
            println!("{:?}", self.is_in_object);
            println!("{:?}", self.current_pos);
            let line = Line::raw(text_to_add.to_string());
            let line = if should_color {
                line.bg(Color::Blue).black()
            } else {
                line
            };
            self.inner.push(line);
            self.internal_buf.drain(..newline_idx + 1);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

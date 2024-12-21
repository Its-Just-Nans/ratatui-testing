use std::collections::{HashMap, VecDeque};

use crate::json::JsonContainer;

#[derive(Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonEdition {
    Key(String),
    Index(usize),
}

#[derive(Default)]
pub struct App<'a> {
    pub input_file: Option<String>,
    pub key_input: String,              // the currently being edited json key.
    pub value_input: String,            // the currently being edited json value.
    pub pairs: HashMap<String, String>, // The representation of our key and value pairs with serde Serialize support
    pub json_container: JsonContainer<'a>,
    pub ref_edition: VecDeque<JsonEdition>,
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub currently_editing: Option<CurrentlyEditing>, // the optional state containing which of the key or value pair the user is editing. It is an option, because when the user is not directly editing a key-value pair, this will be set to `None`.
    phantom: std::marker::PhantomData<&'a ()>,
}

impl App<'_> {
    pub fn new(input_file: Option<String>) -> Self {
        let default_json = match input_file.clone() {
            Some(input_file) => {
                let json = std::fs::read_to_string(input_file).expect("Could not read file");
                serde_json::from_str(&json).expect("Could not parse json")
            }
            None => serde_json::Value::Null,
        };
        let mut vec = VecDeque::new();
        vec.push_back(JsonEdition::Key("array".to_string()));
        vec.push_back(JsonEdition::Index(0));
        Self {
            input_file,
            json_container: JsonContainer::new(default_json),
            ref_edition: vec,
            ..Default::default()
        }
    }

    pub fn save_key_value(&mut self) {
        self.pairs
            .insert(self.key_input.clone(), self.value_input.clone());

        self.key_input = String::new();
        self.value_input = String::new();
        self.currently_editing = None;
    }

    pub fn toggle_editing(&mut self) {
        if let Some(edit_mode) = &self.currently_editing {
            match edit_mode {
                CurrentlyEditing::Key => self.currently_editing = Some(CurrentlyEditing::Value),
                CurrentlyEditing::Value => self.currently_editing = Some(CurrentlyEditing::Key),
            };
        } else {
            self.currently_editing = Some(CurrentlyEditing::Key);
        }
    }

    pub fn write_output(&self) {
        match &self.input_file {
            Some(_) => {
                self.write_json().expect("Could not write json");
            }
            None => {
                self.print_json().expect("Could not print json");
            }
        }
    }

    pub fn print_json(&self) -> serde_json::Result<()> {
        let output = serde_json::to_string(&self.pairs)?;
        println!("{}", output);
        Ok(())
    }

    pub fn write_json(&self) -> serde_json::Result<()> {
        let output = serde_json::to_string(&self.pairs).expect("Could not serialize json");
        let output_file = self.input_file.clone().unwrap_or("output.json".to_string());
        std::fs::write(output_file, output).expect("Could not write to file");
        Ok(())
    }

    pub fn render_json(&mut self) {
        self.json_container.create_lines(self.ref_edition.clone());
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvent {
    pub name: String,
    pub description: String,
    pub creator: Option<String>,
}

impl NewEvent {
    pub fn set_creator(&mut self, creator: String) {
        self.creator = Option::Some(creator);
    }
}

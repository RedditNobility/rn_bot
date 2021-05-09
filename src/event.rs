use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

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

impl Display for NewEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

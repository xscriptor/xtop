use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub palette: [[u8; 3]; 16],
}

impl Theme {
    pub fn bg(&self) -> &[u8; 3] {
        &self.palette[0]
    }

    pub fn fg(&self) -> &[u8; 3] {
        &self.palette[7]
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub f_name: String,
    pub l_name: String,
    pub email: String,
}

impl User {
    pub fn new(f_name: &str, l_name: &str, email: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: format!("{} {}", f_name, l_name),
            f_name: f_name.into(),
            l_name: l_name.into(),
            email: email.into(),
        }
    }
}
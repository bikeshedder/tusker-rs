use std::collections::HashMap;

use super::{table::Table, view::View};

#[derive(Debug, Default)]
pub struct Schema {
    pub name: String,
    pub tables: HashMap<String, Table>,
    pub views: HashMap<String, View>,
}

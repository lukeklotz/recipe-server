use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Recipe {
    pub title: String,
    pub ingredients: Vec<String>,
}

pub fn get_recipe() -> Recipe {
    let recipe = Recipe {
        title: "PB-n-J".to_string(),
        ingredients: vec!["Bread".into(), "Peanut Butter".into(), "Jelly".into()]
    };
    
    recipe
}


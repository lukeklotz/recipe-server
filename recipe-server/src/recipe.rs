//"https://dummyjson.com/recipes/{}, i"
//"i" is used used to fetch random recipe
//limit between 0 - 29

use sqlx::{
    migrate::MigrateDatabase, 
    sqlite::SqliteQueryResult, 
    Sqlite, 
    SqlitePool,
    Row,
};


use rand::prelude::IndexedRandom;
use std::fs;
use serde::Deserialize;

pub const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Deserialize)]
pub struct Recipe {
    pub title: String,
    pub ingredients: Vec<String>,
}


// parses json into a vector of Recipe structs
pub fn get_recipe() -> Vec<Recipe> {

    // Read the JSON file
    let json = fs::read_to_string("recipes.json").expect("Failed to read recipes.json");

    // Parse JSON into a Vec<Recipe>
    let recipes: Vec<Recipe> =
        serde_json::from_str(&json).expect("Failed to parse JSON");

    recipes
}

pub async fn create_db() -> Result<(), sqlx::Error> {

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    Ok(())
}

pub async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create recipes table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recipes (
            id INTEGER PRIMARY KEY NOT NULL,
            name VARCHAR(250) NOT NULL
        );"
    )
    .execute(pool)
    .await?;

    // Create ingredients table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ingredients (
            id INTEGER PRIMARY KEY NOT NULL,
            recipe_id INTEGER NOT NULL,
            name VARCHAR(250) NOT NULL,
            FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
        );"
    )
    .execute(pool)
    .await?;

    Ok(())
}



pub async fn insert_recipe(pool: &SqlitePool, recipe: &Recipe) -> Result<(), sqlx::Error> {
    
    // Start a transaction to ensure consistency
    let mut tx = pool.begin().await?;

    // Insert into recipes table and get the ID
    let recipe_id: i64 = sqlx::query(
        "INSERT INTO recipes (name) VALUES (?) RETURNING id;"
    )
    .bind(&recipe.title)
    .fetch_one(&mut *tx)
    .await?
    .get("id");

    // Insert each ingredient
    for ingredient in &recipe.ingredients {
        sqlx::query(
            "INSERT INTO ingredients (recipe_id, name) VALUES (?, ?);"
        )
        .bind(recipe_id)
        .bind(ingredient)
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}



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

use std::io::Error;
use rand::prelude::IndexedRandom;
use std::fs;
use serde::Deserialize;

pub const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Deserialize, Debug)]
pub struct Recipe {
    pub id: i64,
    pub title: String,
    pub ingredients: Vec<String>,
}


// parses json into a vector of Recipe structs
pub fn get_recipe() -> Vec<Recipe> {

    let path = "recipes.json";

    println!("Trying to read: {:?}", path);

    let json = match fs::read_to_string(path) {
        Ok(json_to_parse) => {
            println!("Successfully read recipes.json! Content length: {}", json_to_parse.len());
            json_to_parse
        }
        Err(e) => {
            println!("Failed to read recipes.json: {:#?}", e);
            return vec![];
        }
    };

    let recipes: Vec<Recipe> = match serde_json::from_str(&json) {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to parse JSON: {:#?}", e);
            return vec![];
        }
    };

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

pub async fn insert(pool: &SqlitePool, recipes: &Vec<Recipe>) -> Result<(), sqlx::Error> {

    println!("Inserting recipes into db...");

    for recipe in recipes {
        println!("Inserting recipe: {:?}", recipe.title);
        insert_recipe(pool, recipe).await?;
    }
    Ok(())
}


async fn insert_recipe(pool: &SqlitePool, recipe: &Recipe) -> Result<(), sqlx::Error> {
    
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


pub async fn query_random_recipe(pool: &SqlitePool) -> Result<Recipe, sqlx::Error> {
    let row = sqlx::query("SELECT id, name FROM recipes ORDER BY RANDOM() LIMIT 1")
        .fetch_one(pool)
        .await?;

    let recipe_id: i64 = row.get("id");
    let recipe_name: String = row.get("name");

    let ingredient_rows = sqlx::query("SELECT name FROM ingredients WHERE recipe_id = ?")
        .bind(recipe_id)
        .fetch_all(pool)
        .await?;

    let ingredients: Vec<String> = ingredient_rows
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();

    Ok(Recipe {
        id: recipe_id,
        title: recipe_name,
        ingredients,
    })
}


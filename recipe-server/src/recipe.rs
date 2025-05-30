//"https://dummyjson.com/recipes/{}, i"
//"i" is used used to fetch random recipe
//limit between 0 - 29
use crate::IndexTemplate;

use sqlx::{
    migrate::MigrateDatabase, 
    //sqlite::SqliteQueryResult, 
    Sqlite, 
    SqlitePool,
    Row,
    Error,
};

//use std::io::Error;
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
pub fn get_recipes() -> Vec<Recipe> {

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

pub async fn create_db() -> Result<bool, sqlx::Error> {

    match Sqlite::database_exists(DB_URL).await {

        //db does not exist
        Ok(false) => {
            println!("Creating database {}", DB_URL);
            Sqlite::create_database(DB_URL).await?;
            println!("Create db success");
            Ok(true) 
        }

        //db does exist
        Ok(true) => {
            println!("Database already exists");
            Ok(false)
        }       

        Err(e) => {
            eprintln!("something terrible has just happened");
            Err(e)
        }
    }
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

    //println!("recipe_id at insert_recipe: {}", recipe_id);

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

pub async fn query_next_recipe(pool: &SqlitePool, mut recipe_index: i64) -> Result<Recipe, sqlx::Error> {
    let row = sqlx::query("SELECT id, name FROM recipes WHERE id > ? ORDER BY id ASC LIMIT 1")
        .bind(recipe_index)
        .fetch_optional(pool)
        .await?;

    let row = match row {
        Some(row) => row,
        None => {
            sqlx::query("SELECT id, name FROM recipes ORDER BY id ASC LIMIT 1")
                .fetch_one(pool)
                .await?
        }
    };

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

pub async fn query_prev_recipe(pool: &SqlitePool, recipe_index: i64) -> Result<Recipe, sqlx::Error> {
    let row = sqlx::query("SELECT id, name FROM recipes WHERE id < ? ORDER BY id DESC LIMIT 1")
        .bind(recipe_index)
        .fetch_optional(pool)
        .await?;

    let row = match row {
        Some(row) => row,
        None => {
            sqlx::query("SELECT id, name FROM recipes ORDER BY id DESC LIMIT 1")
                .fetch_one(pool)
                .await?
        }
    };

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

//fn get_current_recipe_id()

#[derive(Deserialize, Debug)]
pub struct RecipeNavigator {
    pub direction: String,
    pub current_id: Option<i64>
}

pub async fn query_recipe(pool: &SqlitePool, nav: RecipeNavigator, recipe_index: i64) -> Result<Recipe, Error> {
    match nav.direction.as_str() {
        "prev" => {
            println!("prev");
            query_prev_recipe(pool, recipe_index).await
        }
        "next" => {
            println!("next");

            query_next_recipe(pool, recipe_index).await
        }
        "random" => {
            println!("random");
            query_random_recipe(pool).await
        }
        _ => Err(Error::RowNotFound),
    }
}


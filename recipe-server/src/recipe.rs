//"https://dummyjson.com/recipes/{}, i"
//"i" is used used to fetch random recipe
//limit between 0 - 29

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, Sqlite, SqlitePool};

use serde::Deserialize;

pub const DB_URL: &str = "sqlite://sqlite.db";

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

pub async fn create_db() -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    //create recipes
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS recipes 
                                            (id INTEGER PRIMARY KEY NOT NULL, 
                                            name VARCHAR(250) NOT NULL);")
                                            .execute(&db)
                                            .await
                                            .unwrap();
    println!("Create user table result: {:?}", result);

    Ok(result)
}

pub async fn insert_recipe(pool: &SqlitePool, recipe: &Recipe) -> Result<SqliteQueryResult, sqlx::Error> {
    let result = sqlx::query("INSERT INTO recipes (name) VALUES (?)")
        .bind(&recipe.title)
        .execute(pool)
        .await?;

    println!("Inserted recipe: {:?}", result);
    Ok(result)
}


// fn loadRecipe() {
// recipe = jsonFile[randomIndex]
// let recipe = Recipe {
//      title: recipe.title
//      ingredients: append recipe ingredients somehow.. ?
//  }
//}
// recipe
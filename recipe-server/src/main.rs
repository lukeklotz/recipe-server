mod recipe;
mod templates;

use recipe::*;
use templates::*;
use askama::Template;

use sqlx::{SqlitePool};

//use serde::Deserialize;

use axum::{
    extract::{Form, State},
    routing::{get, post},
    response,
    response::Html,
    Router,
};

async fn render_recipe_page(
    State(pool): State<SqlitePool>, 
    Form(nav): Form<RecipeNavigator>) 
    -> response::Html<String> {

    let recipe = query_recipe(&pool, nav).await.unwrap();

    println!("recipe: {:?}", recipe);

    let template = IndexTemplate::recipe(&recipe);

    Html(template.render().unwrap())
}

async fn render_index(State(pool): State<SqlitePool>) -> Html<String> {
    
    let recipe = query_random_recipe(&pool).await.unwrap();
    let template = IndexTemplate::recipe(&recipe);

    Html(template.render().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error>{

    let was_created = recipe::create_db().await?;

    let pool = SqlitePool::connect(recipe::DB_URL).await?;

    if was_created {
        println!("Populating db...");
        recipe::create_tables(&pool).await?;
        let recipes = recipe::get_recipes(); 
        recipe::insert(&pool, &recipes).await?;
    }
   
    let app = Router::new()
        .route("/recipe", post(render_recipe_page))
        .route("/", get(render_index))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
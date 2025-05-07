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
    Form(data): Form<RecipeNavigator>) 
    -> response::Html<String> {

    //TODO: query_recipe returns a string but is supposed to return a recipe struct
    let recipe = query_recipe(&pool, data).await.unwrap();

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

    let _result = recipe::create_db().await;

    // Create a connection pool
    let pool = SqlitePool::connect(recipe::DB_URL).await?;

    recipe::create_tables(&pool).await?;

    let recipe = recipe::get_recipe(); 
    recipe::insert(&pool, &recipe).await?;
    
    println!("here");
   
    let app = Router::new()
        .route("/recipe", post(render_recipe_page))
        .route("/", get(render_index))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
mod recipe;
mod templates;

use recipe::*;
use templates::*;
use askama::Template;

use sqlx::{SqlitePool};

use axum::{
    routing::get,
    response::Html,
    Router,
};


//TODO: render HTML from database instead of directly from struct

/* 
async fn render_recipe_page() -> Html<String> {

    //build recipe struct from db query

}
*/

#[tokio::main]
async fn main() -> Result<(), sqlx::Error>{

    let _result = recipe::create_db().await;

    // Create a connection pool
    let pool = SqlitePool::connect(recipe::DB_URL).await?;

    create_tables(&pool).await?;

    let recipe = recipe::get_recipe(); 
    recipe::insert(&pool, &recipe).await?;
   
    //TODO: Query DB
    //TODO: Render recipe page


    let app = Router::new()
                    .route("/", get("replace me with render recipe page"))
                    .route("/other", get(|| async { "other, world!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
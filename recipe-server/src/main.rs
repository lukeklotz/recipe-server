mod recipe;
mod templates;

use recipe::*;
use templates::*;
use askama::Template;

use axum::{
    routing::get,
    response::Html,
    Router,
};

async fn render_recipe_page() -> Html<String> {
    let recipe = recipe::get_recipe();
    let template = IndexTemplate::recipe(&recipe);

    Html(template.render().unwrap())
}

#[tokio::main]
async fn main() {


    let app = Router::new()
                    .route("/", get(render_recipe_page))
                    .route("/other", get(|| async { "other, world!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
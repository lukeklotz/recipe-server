mod recipe;
mod templates;

use recipe::*;
use templates::*;
use askama::Template;
use utoipa::path;
use sqlx::{SqlitePool};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use axum::{
    extract::{Form, Path, State},
    routing::{get, post},
    response::{Html, Json, IntoResponse},
    http::StatusCode,
    Router,
};

//rest api functions

#[utoipa::path(
    get,
    path = "/api/recipe/random",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn api_random(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let recipe = recipe::query_random_recipe(&pool).await;

    match recipe {
        Ok(recipe) => {
            let template = IndexTemplate::recipe(&recipe);
            Html(template.render().unwrap()).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/recipe/{id}",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 404, description = "Recipe not found")
    ),
    params(
        ("id" = i64, Path, description = "Recipe database ID")
    )
)]
pub async fn api_id(State(pool): State<SqlitePool>, Path(current_id): Path<i64>) -> impl IntoResponse {
    let recipe = recipe::query_recipe_by_id(&pool, current_id).await;

    match recipe {
        Ok(recipe) => {
            let template = IndexTemplate::recipe(&recipe);
            Html(template.render().unwrap()).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
// rest api functions end

// OpenAPI start
#[derive(OpenApi)]
#[openapi(
    paths(
        api_id,
        api_random, 
    ),
    components(
        schemas(Recipe)
    ),
    tags(
        (name = "Recipes", description = "Recipe-related endpoints")
    )
)]

pub struct ApiDoc;
//Open API end

//render functions
async fn render_recipe_page(
    State(pool): State<SqlitePool>, 
    Form(nav): Form<RecipeNavigator>) 
    -> Result<Html<String>, StatusCode>  {

    let current_id = nav.current_id.unwrap_or(0);

    println!("current id: {}", current_id);

    let recipe = match nav.direction.as_str() {
        "random" => query_random_recipe(&pool).await.unwrap(),
        "next" | "prev" => {
            if current_id == 0 {
                query_random_recipe(&pool).await.unwrap()
            } else {
                query_recipe(&pool, nav, current_id).await.unwrap()
            }
        }
        _ => {
            query_random_recipe(&pool).await.unwrap()
        }
    };
    println!("recipe: {:?}", recipe);
    let template = IndexTemplate::recipe(&recipe);

    Ok(Html(template.render().unwrap()))
}

async fn render_index(State(pool): State<SqlitePool>) -> Html<String> {
    
    let recipe = query_random_recipe(&pool).await.unwrap();
    let template = IndexTemplate::recipe(&recipe);

    Html(template.render().unwrap())
}
//render functions end

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
        .route("/", get(render_index))
        .route("/recipe", post(render_recipe_page))
        .route("/api/recipe/random", get(api_random))
        .route("/api/recipe/{id}", get(api_id))
        .with_state(pool.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
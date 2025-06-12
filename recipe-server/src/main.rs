mod recipe;
mod templates;

use axum::extract::Query;
use std::sync::Arc;
use axum::Extension;
use recipe::*;
use templates::*;
use askama::Template;
use sqlx::SqlitePool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::{
    extract::{Form, Path},
    routing::{get, post},
    response::{Html, IntoResponse, Json},
    http::{StatusCode, HeaderMap, HeaderValue, Method},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tokio::net;

// JSON API functions 
//
//GET random
#[utoipa::path(
    get,
    path = "/api/recipe/random",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn api_random_json(Extension(pool): Extension<Arc<SqlitePool>>) -> impl IntoResponse {
    let recipe = recipe::query_random_recipe(&pool).await;

    match recipe {
        Ok(recipe) => Json(recipe).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}


//GET next
#[utoipa::path(
    get,
    path = "/api/recipe/next",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 500, description = "Internal server error")
    ),
    params (
        ("id" = i64, Query, description = "Recipe database ID")
    )
)]
pub async fn api_next_json(Extension(pool): Extension<Arc<SqlitePool>>,
                           Query(nav): Query<RecipeNavigator>) -> impl IntoResponse {
    
    let current_id = nav.current_id.unwrap_or(1);

    println!("current_id: {}", current_id);

    let recipe = recipe::query_recipe(&pool, nav).await;

    match recipe {
        Ok(recipe) => Json(recipe).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

//GET prev
#[utoipa::path(
    get,
    path = "/api/recipe/prev",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 500, description = "Internal server error")
    ),
    params (
        ("id" = i64, Query, description = "Recipe database ID")
    )
)]
pub async fn api_prev_json(Extension(pool): Extension<Arc<SqlitePool>>,
                           Query(nav): Query<RecipeNavigator>) -> impl IntoResponse {
    
    let current_id = nav.current_id.unwrap_or(1);
    println!("current_id: {}", current_id); //for debugging..

    let recipe = recipe::query_recipe(&pool, nav).await;

    match recipe {
        Ok(recipe) => Json(recipe).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

//GET by id
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
pub async fn api_id_json(Extension(pool): Extension<Arc<SqlitePool>>, Path(current_id): Path<i64>) -> impl IntoResponse {
    let recipe = recipe::query_recipe_by_id(&pool, current_id).await;

    match recipe {
        Ok(recipe) => Json(recipe).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// HTML API functions
#[utoipa::path(
    get,
    path = "/api/recipe/random/html",
    responses(
        (status = 200, description = "Recipe found", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn api_random_html(Extension(pool): Extension<Arc<SqlitePool>>) -> impl IntoResponse {
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
    path = "/api/recipe/{id}/html",
    responses(
        (status = 200, description = "Recipe found", body = String),
        (status = 404, description = "Recipe not found")
    ),
    params(
        ("id" = i64, Path, description = "Recipe database ID")
    )
)]
pub async fn api_id_html(Extension(pool): Extension<Arc<SqlitePool>>, Path(current_id): Path<i64>) -> impl IntoResponse {
    let recipe = recipe::query_recipe_by_id(&pool, current_id).await;

    match recipe {
        Ok(recipe) => {
            let template = IndexTemplate::recipe(&recipe);
            Html(template.render().unwrap()).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        api_id_json,
        api_random_json,
        api_id_html,
        api_random_html,
    ),
    components(
        schemas(Recipe)
    ),
    tags(
        (name = "Recipes", description = "Recipe-related endpoints")
    )
)]
pub struct ApiDoc;

// Render functions for existing web interface
async fn render_recipe_page(
    Extension(pool): Extension<Arc<SqlitePool>>, 
    Form(nav): Form<RecipeNavigator>) 
    -> Result<Html<String>, StatusCode>  {
    
    //this is really bad but oh well
    let current_id = nav.current_id.unwrap_or(0);

    let recipe = match nav.direction.as_str() {
        "random" => query_random_recipe(&pool).await.unwrap(),
        "next" | "prev" => {
            if current_id == 0 {
                query_random_recipe(&pool).await.unwrap()
            } else {
                query_recipe(&pool, nav).await.unwrap()
            }
        }
        _ => {
            query_random_recipe(&pool).await.unwrap()
        }
    };

    let template = IndexTemplate::recipe(&recipe);
    Ok(Html(template.render().unwrap()))
}

async fn render_index(Extension(pool): Extension<Arc<SqlitePool>>) -> Result<Html<String>, StatusCode> {
    let recipe = query_random_recipe(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let template = IndexTemplate::recipe(&recipe);
    let html = template.render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let was_created = recipe::create_db().await?;
    let pre_arc_pool = SqlitePool::connect(recipe::DB_URL).await?;
    let pool = Arc::new(pre_arc_pool);

    if was_created {
        println!("Populating db...");
        recipe::create_tables(&pool).await?;
        let recipes = recipe::get_recipes(); 
        recipe::insert(&pool, &recipes).await?;
    }

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    // Configure CORS 
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap()) 
        .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        // Existing HTML routes
        .route("/", get(render_index))
        .route("/recipe", post(render_recipe_page))
        
        // JSON API routes Leptos frontend
        .route("/api/recipe/random", get(api_random_json))
        .route("/api/recipe/next", get(api_next_json))
        .route("/api/recipe/prev", get(api_prev_json))
        .route("/api/recipe/{id}", get(api_id_json))
        
        // HTML API routes mostly for testing
        //.route("/api/recipe/random/html", get(api_random_html))
        //.route("/api/recipe/{id}/html", get(api_id_html))
        
        .merge(swagger_ui)
        .layer(Extension(pool))
        .layer(cors); 

    console_error_panic_hook::set_once();

    println!("Server starting on http://0.0.0.0:3000");
    let listener = net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
mod recipe;
mod templates;

use std::{net::SocketAddr, sync::Arc};
use axum::Extension;
use recipe::*;
use templates::*;
use askama::Template;
use sqlx::{SqlitePool};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa::openapi::Server;
use axum::{
    extract::{Form, Path, State},
    routing::{get, post},
    response::{Html, IntoResponse},
    http::StatusCode,
    Router,
};
use tokio::net;

//rest api functions
#[utoipa::path(
    get,
    path = "/api/recipe/random",
    responses(
        (status = 200, description = "Recipe found", body = Recipe),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn api_random(Extension(pool): Extension<Arc<SqlitePool>>) -> impl IntoResponse {
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
pub async fn api_id(Extension(pool): Extension<Arc<SqlitePool>>, Path(current_id): Path<i64>) -> impl IntoResponse {

    //get the currect recipe thats being displayed
    let recipe = recipe::query_recipe_by_id(&pool, current_id).await;

    match recipe {
        Ok(recipe) => {
            //create a template and render it
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
    Extension(pool): Extension<Arc<SqlitePool>>, 
    Form(nav): Form<RecipeNavigator>) 
    -> Result<Html<String>, StatusCode>  {
    
    //When the default home page is loaded for the first time
    //there is no ID associated with the nav component
    //so this checks if thats the case and handles it later
    //WARNING: This is hacky and not really correct, but works for now
    let current_id = nav.current_id.unwrap_or(0);

    //nav.direction is assigned by html buttons
    //current options are "random, next, prev"
    let recipe = match nav.direction.as_str() {

        //call a query type based on the value of nav.direction
        //query result assigned to recipe
        "random" => query_random_recipe(&pool).await.unwrap(),
        "next" | "prev" => {
            if current_id == 0 {
                //this is whats triggered on the initial page load
                query_random_recipe(&pool).await.unwrap()
            } else {
                query_recipe(&pool, nav, current_id).await.unwrap()
            }
        }
        _ => {
            query_random_recipe(&pool).await.unwrap()
        }
    };
    //get our template from our recipe struct
    let template = IndexTemplate::recipe(&recipe);

    //return the rendered template
    Ok(Html(template.render().unwrap()))
}

async fn render_index(Extension(pool): Extension<Arc<SqlitePool>>) -> Result<Html<String>, StatusCode> {
    //this is called on inital load 
    //since theres no current index, we load a random one
    let recipe = query_random_recipe(&pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let template = IndexTemplate::recipe(&recipe);
    let html = template.render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}
//render functions end

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

    let app = Router::new()
        .route("/", get(render_index))
        .route("/recipe", post(render_recipe_page))
        .route("/api/recipe/random", get(api_random))
        .route("/api/recipe/{id}", get(api_id))
        .merge(swagger_ui)
        .layer(Extension(pool));

    let listener = net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
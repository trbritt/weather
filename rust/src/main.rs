use std::net::SocketAddr;
use axum::{routing::get, Router, extract::{Query, State}};
use anyhow::Context;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
};
// use axum_macros;
mod users;
use crate::users::User;
mod utils;
use crate::utils::*;


async fn fetch_lat_long(city: &str) -> Result<LatLong, anyhow::Error> {
	let endpoint = format!(
    	"https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
    	city
	);
	let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
	response.results.get(0).cloned().context("No results found")
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, anyhow::Error> {
	let endpoint = format!(
    	"https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
    	lat_long.latitude, lat_long.longitude
	);
	let response = reqwest::get(&endpoint).await?.json::<WeatherResponse>().await?;
	Ok(response)
}

// the above two methods are what query the interwebs for data, but we can be smarter about it if we 
// use caching to see if the city has already been queried from the internet
async fn get_lat_long(pool: &PgPool, name: &str) -> Result<LatLong, anyhow::Error> {
    let lat_long = sqlx::query_as::<_,LatLong>(
        "SELECT latitude,longitude FROM cities WHERE name=$1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if let Some(lat_long) = lat_long {
        // println!("We found something");
    	return Ok(lat_long);
	}

    let lat_long = fetch_lat_long(name).await?;
    sqlx::query("INSERT INTO cities (name, latitude, longitude) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;
    Ok(lat_long)

}
//the problem with city: string below is that the String type is not a valid extractor 
//for the data. We need to use an extractor that deserializes query strings into some type.
// it is a template, whose specialization
//is a hashmap of key, value pairs. In this case, both key and value are of type String
// We can either make this explicit via the type Query<HashMap<String,String>>, but
// this is done automatically if we defined a struct with the deserialized. See comments in utils
// #[axum_macros::debug_handler]
async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>
) -> Result<WeatherDisplay, AppError> {
	// let lat_long = fetch_lat_long(&params.city).await?;
    let lat_long = get_lat_long(&pool, &params.city).await?;
	let weather = fetch_weather(lat_long).await?;
	Ok(WeatherDisplay::new(params.city, weather))
}

async fn get_last_cities(pool: &PgPool) -> Result<Vec<City>, AppError> {
    let cities = sqlx::query_as::<_, City>("SELECT name FROM cities ORDER BY id DESC LIMIT 10")
        .fetch_all(pool)
        .await?;
    Ok(cities)
}
//basic handler that repsonds with a static string
// #[axum_macros::debug_handler]
async fn index() -> IndexTemplate {
	IndexTemplate
}
async fn stats(_user: User, State(pool): State<PgPool>) -> Result<StatsTemplate, AppError> {
	let cities = get_last_cities(&pool).await?;
	Ok(StatsTemplate { cities })
}

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> anyhow::Result<()>{
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = sqlx::PgPool::connect(&db_connection_str)
        .await
        .context("can't connect to database")?;
    let api_router = Router::new().route("/hello", get(hello_from_the_server));

    let assets_path = std::env::current_dir().unwrap();
    let app = Router::new()
        .nest("/api", api_router)
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        )      
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = SocketAddr::from(([127,0,0,1],8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}

use std::{net::SocketAddr};
use axum::{routing::get, Router, extract::{Query, State}};
use anyhow::Context;
use sqlx::PgPool;
// use axum_macros;
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
	let lat_long = fetch_lat_long(&params.city).await?;
	let weather = fetch_weather(lat_long).await?;
	Ok(WeatherDisplay::new(params.city, weather))
}


//basic handler that repsonds with a static string
// #[axum_macros::debug_handler]
async fn index() -> IndexTemplate {
	IndexTemplate
}
async fn stats() -> &'static str {
    "Stats"
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = sqlx::PgPool::connect(&db_connection_str)
        .await
        .context("can't connect to database")?;
    
    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(pool);

    let addr = SocketAddr::from(([127,0,0,1],8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

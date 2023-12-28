use axum::response::{IntoResponse, Response, Html};
use askama::Template;
use reqwest::StatusCode;
use serde::Deserialize;
use anyhow;
// use sqlx;
// pub fn into_response<T: Template>(t: &T) -> Response {
//     match t.render() {
//         Ok(body) => {
//             let headers = [(
//                 http::header::CONTENT_TYPE,
//                 http::HeaderValue::from_static(T::MIME_TYPE),
//             )];

//             (headers, body).into_response()
//         }
//         Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
//     }
// }
// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
	fn into_response(self) -> Response {
    	(
        	StatusCode::INTERNAL_SERVER_ERROR,
        	format!("Something went wrong: {}", self.0),
    	)
        	.into_response()
	}
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
    	Self(err.into())
	}
}

//lets define the structs as received from the api
#[derive(Deserialize, Debug, Clone)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

//In comparison to Go, we don't use tags to specify the field names. 
//Instead, we use the #[derive(Deserialize)] attribute from serde to 
//automatically derive the Deserialize trait for our structs. 
//These derive macros are very powerful and allow us to do a lot of 
//things with very little code, including handling parsing errors for our types
#[derive(Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
	pub latitude: f64,
	pub longitude: f64,
	pub timezone: String,
	pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
	pub time: Vec<String>,
	pub temperature_2m: Vec<f64>,
}

#[derive(Template,Deserialize, Debug)]
#[template(path = "weather.html")]
pub struct WeatherDisplay {
	pub city: String,
	pub forecasts: Vec<Forecast>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;


#[derive(Deserialize, Debug)]
pub struct Forecast {
	pub date: String,
	pub temperature: String,
}

impl WeatherDisplay {
    pub fn new(city: String, response: WeatherResponse) -> Self {
        let display = WeatherDisplay {
            city,
            forecasts: response
                .hourly
                .time
                .iter()
                .zip(response.hourly.temperature_2m.iter())
                .map(|(date, temperature)| Forecast {
                    date:   date.to_string(),
                    temperature: temperature.to_string()
                })
                .collect(),
        };
        display
    }
}

impl IntoResponse for WeatherDisplay {
    fn into_response(self) -> Response {
        let body = Html(self.render().unwrap()); // Use the render method from the askama template
        body.into_response()
    }
}
impl IntoResponse for IndexTemplate {
    fn into_response(self) -> Response {
        let body = Html(self.render().unwrap()); // Use the render method from the askama template
        body.into_response()
    }
}
// below is a version of the weather handler that uses a mix of parsing and handler logic.
// this is not very rust-y, so instead we opt to do the parsing logic in the constructor of 
// the Weatherdisplay struct! See above
// async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, StatusCode> {
//     let lat_long = fetch_lat_long(&params.city)
//         .await
//         .map_err(|_| StatusCode::NOT_FOUND)?;
//     let weather = fetch_weather(lat_long)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     let display = WeatherDisplay {
//         city: params.city,
//         forecasts: weather
//             .hourly
//             .time
//             .iter()
//             .zip(weather.hourly.temperature_2m.iter())
//             .map(|(date, temperature)| Forecast {
//                 date: date.to_string(),
//                 temperature: temperature.to_string(),
//             })
//             .collect(),
//     };
//     Ok(format!("{:?}", display))
// }

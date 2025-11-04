// src/weather/mod.rs

pub mod api;
pub mod models;
pub mod components;
pub mod test_data;  // Test data link

pub use api::EnvironmentCanadaClient;
pub use models::WeatherData;
pub use components::WeatherDisplay;
// src/weather/mod.rs
pub mod api;
pub mod components;

// Re-export the main types that other modules need
pub use api::{
    WeatherData, 
    CurrentConditions, 
    HourlyForecast, 
    DailyForecast,
    AirQuality,
    fetch_weather_data,
};

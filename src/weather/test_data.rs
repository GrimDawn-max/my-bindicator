// src/weather/test_data.rs

use crate::weather::models::*;

pub fn get_mock_weather() -> WeatherData {
    WeatherData {
        location: "Toronto".to_string(),
        current: CurrentConditions {
            temperature: 8.0,
            condition: "Mainly Cloudy".to_string(),
            humidity: Some(68),
            pressure: Some(101.3),
            visibility: Some(24.0),
            wind_speed: Some(15),
            wind_direction: Some("SW".to_string()),
            wind_chill: Some(6.0),
            humidex: None,
        },
        forecasts: vec![
            DailyForecast {
                day_name: "Today".to_string(),
                high: Some(12),
                low: Some(6),
                summary: "Cloudy periods".to_string(),
                pop: Some(30),
                icon: "☁️".to_string(),
            },
            DailyForecast {
                day_name: "Monday".to_string(),
                high: Some(10),
                low: Some(2),
                summary: "Sunny".to_string(),
                pop: Some(10),
                icon: "☀️".to_string(),
            },
            DailyForecast {
                day_name: "Tuesday".to_string(),
                high: Some(11),
                low: Some(4),
                summary: "Mix sun and cloud".to_string(),
                pop: Some(20),
                icon: "⛅".to_string(),
            },
        ],
        warnings: vec![],
        last_updated: "Mock Data".to_string(),
    }
}
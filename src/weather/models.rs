// src/weather/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WeatherData {
    pub location: String,
    pub current: CurrentConditions,
    pub forecasts: Vec<DailyForecast>,
    pub warnings: Vec<WeatherWarning>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CurrentConditions {
    pub temperature: f32,
    pub condition: String,
    pub humidity: Option<u8>, 
    pub pressure: Option<f32>,
    pub visibility: Option<f32>,
    pub wind_speed: Option<u32>,
    pub wind_direction: Option<String>,
    pub wind_chill: Option<f32>,
    pub humidex: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DailyForecast {
    pub day_name: String,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub summary: String,
    pub pop: Option<u32>, // Probability of precipitation
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WeatherWarning {
    pub warning_type: String,
    pub priority: String,
    pub description: String,
}

impl CurrentConditions {
    pub fn feels_like(&self) -> f32 {
        self.wind_chill
            .or(self.humidex)
            .unwrap_or(self.temperature)
    }
    
    pub fn wind_description(&self) -> String {
        match (&self.wind_direction, self.wind_speed) {
            (Some(dir), Some(speed)) => format!("{} {} km/h", dir, speed),
            (Some(dir), None) => dir.clone(),
            (None, Some(speed)) => format!("{} km/h", speed),
            (None, None) => "Calm".to_string(),
        }
    }
}

impl DailyForecast {
    pub fn get_emoji(summary: &str) -> String {
        let s = summary.to_lowercase();
        
        if s.contains("sunny") || s.contains("clear") {
            "☀️".to_string()
        } else if s.contains("partly cloudy") || s.contains("mix") {
            "⛅".to_string()
        } else if s.contains("cloud") {
            "☁️".to_string()
        } else if s.contains("rain") || s.contains("shower") {
            "🌧️".to_string()
        } else if s.contains("snow") {
            "🌨️".to_string()
        } else if s.contains("storm") {
            "⛈️".to_string()
        } else {
            "🌤️".to_string()
        }
    }
}

impl WeatherData {
    /// Get forecast for a specific day (useful for bin collection days)
    pub fn get_forecast_for_day(&self, day_name: &str) -> Option<&DailyForecast> {
        self.forecasts
            .iter()
            .find(|f| f.day_name.to_lowercase().contains(&day_name.to_lowercase()))
    }
    
    /// Check if there are any severe weather warnings
    #[allow(dead_code)] // Public API method
    pub fn has_severe_warnings(&self) -> bool {
        self.warnings.iter().any(|w| w.priority == "high")
    }
}

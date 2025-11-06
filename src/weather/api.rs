// src/weather/api.rs

use gloo_net::http::Request;
use roxmltree::Document;
use crate::weather::models::*;
use gloo_console::{log, warn};

pub struct EnvironmentCanadaClient {
    site_code: String,
    province: String,
}

impl EnvironmentCanadaClient {
    pub fn toronto() -> Self {
        Self {
            site_code: "on-143".to_string(),
            province: "ON".to_string(),
        }
    }
    
    /// Fetch current weather data using RSS feed
    pub async fn fetch_weather(&self) -> Result<WeatherData, String> {
        let base_url = format!(
            "https://weather.gc.ca/rss/city/{}_e.xml",
            self.site_code
        );
        
        // Use allorigins proxy
        let url = format!("https://api.allorigins.win/raw?url={}", base_url);
        
        log!("Fetching weather from:", &url);
        
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;
        
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        
        let xml_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {:?}", e))?;
        
        log!("Received XML data, parsing...");
        
        Self::parse_xml(&xml_text)
    }
    
    fn parse_xml(xml_text: &str) -> Result<WeatherData, String> {
        let doc = Document::parse(xml_text)
            .map_err(|e| format!("XML parse error: {}", e))?;
        
        // Find the "Current Conditions" entry
        let current_entry = doc
            .descendants()
            .filter(|n| n.has_tag_name("entry"))
            .find(|entry| {
                entry
                    .descendants()
                    .find(|n| n.has_tag_name("category"))
                    .and_then(|n| n.attribute("term"))
                    .map(|term| term == "Current Conditions")
                    .unwrap_or(false)
            })
            .ok_or("No current conditions entry found")?;
        
        // Parse current conditions from HTML summary
        let summary_html = current_entry
            .descendants()
            .find(|n| n.has_tag_name("summary"))
            .and_then(|n| n.text())
            .ok_or("No summary found")?;
        
        let current = Self::parse_current_from_html(summary_html)?;
        
        // Parse forecasts from forecast entries
        let forecast_entries: Vec<_> = doc
            .descendants()
            .filter(|n| n.has_tag_name("entry"))
            .filter(|entry| {
                entry
                    .descendants()
                    .find(|n| n.has_tag_name("category"))
                    .and_then(|n| n.attribute("term"))
                    .map(|term| term == "Weather Forecasts")
                    .unwrap_or(false)
            })
            .collect();
        
        let forecasts = Self::parse_forecasts_from_entries(&forecast_entries);
        
        // Get location from feed title
        let location = doc
            .descendants()
            .find(|n| n.has_tag_name("title"))
            .and_then(|n| n.text())
            .and_then(|s| s.split(" - ").next())
            .unwrap_or("Toronto")
            .to_string();
        
        // Get update time
        let last_updated = doc
            .descendants()
            .find(|n| n.has_tag_name("updated"))
            .and_then(|n| n.text())
            .unwrap_or("Unknown")
            .to_string();
        
        Ok(WeatherData {
            location,
            current,
            forecasts,
            warnings: vec![],
            last_updated,
        })
    }
    
    fn parse_current_from_html(html: &str) -> Result<CurrentConditions, String> {
        log!("Parsing current conditions from HTML");
        
        // --- Existing Parsers (Temperature, Condition, etc.) ---
        
        // Parse temperature: "<b>Temperature:</b> 8.6&deg;C" or "8.6°C"
        let temp = html
            .split("<b>Temperature:</b>")
            .nth(1)
            .or_else(|| html.split("Temperature:").nth(1))
            .and_then(|s| s.split("&deg;C").next().or_else(|| s.split("°C").next()))
            .and_then(|s| s.trim().parse::<f32>().ok())
            .ok_or("Could not parse temperature")?;
        
        // Parse condition: "<b>Condition:</b> Mainly Sunny<br/>"
        let condition = html
            .split("<b>Condition:</b>")
            .nth(1)
            .or_else(|| html.split("Condition:").nth(1))
            .and_then(|s| s.split("<br/>").next().or_else(|| s.split("<").next()))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Parse humidity: "<b>Humidity:</b> 65 %<br/>"
        // Note: Using parse::<u8>() since we updated models.rs to use u8
        let humidity = html
            .split("<b>Humidity:</b>")
            .nth(1)
            .or_else(|| html.split("Humidity:").nth(1))
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok());
        
        // Parse pressure: "<b>Pressure / Tendency:</b> 102.0 kPa rising<br/>"
        let pressure = html
            .split("<b>Pressure")
            .nth(1)
            .or_else(|| html.split("Pressure").nth(1))
            .and_then(|s| s.split("kPa").next())
            .and_then(|s| s.split_whitespace().last())
            .and_then(|s| s.parse().ok());
        
        // Parse visibility: "<b>Visibility:</b> 24 km<br/>"
        let visibility = html
            .split("<b>Visibility:</b>")
            .nth(1)
            .or_else(|| html.split("Visibility:").nth(1))
            .and_then(|s| s.split("km").next())
            .and_then(|s| s.trim().parse().ok());
        
        // Parse wind: "<b>Wind:</b> W 20 km/h<br/>"
        let wind_text = html
            .split("<b>Wind:</b>")
            .nth(1)
            .or_else(|| html.split("Wind:").nth(1))
            .and_then(|s| s.split("<br/>").next().or_else(|| s.split("<").next()))
            .map(|s| s.trim());
        
        let (wind_direction, wind_speed) = if let Some(wind) = wind_text {
            let parts: Vec<&str> = wind.split_whitespace().collect();
            let dir = parts.get(0).map(|s| s.to_string());
            let speed = parts.get(1).and_then(|s| s.parse().ok());
            (dir, speed)
        } else {
            (None, None)
        };


        // --- NEW AQHI PARSERS ---
        // Expected format: Air Quality Health Index: 3 | Low Risk
        let aqhi_text = html
            .split("Air Quality Health Index:")
            .nth(1)
            .and_then(|s| s.split("<br/>").next());

        let (aqhi_value, aqhi_risk) = if let Some(text) = aqhi_text {
            let parts: Vec<&str> = text.split('|').collect();
            let value = parts.get(0)
                .and_then(|s| s.trim().parse::<u8>().ok());
            let risk = parts.get(1)
                .map(|s| s.trim().to_string());
            (value, risk)
        } else {
            (None, None)
        };


        // --- NEW SUN TIMES PARSERS ---
        // Expected format: Sunrise: 07:44 EDT
        let sunrise = html
            .split("Sunrise:")
            .nth(1)
            .and_then(|s| s.split("<br/>").next())
            .map(|s| s.trim().to_string());

        // Expected format: Sunset: 16:51 EST
        let sunset = html
            .split("Sunset:")
            .nth(1)
            .and_then(|s| s.split("<br/>").next())
            .map(|s| s.trim().to_string());
        
        
        log!("Parsed:", &format!("{}°C, {}", temp, condition));
        
        Ok(CurrentConditions {
            temperature: temp,
            condition,
            humidity,
            pressure,
            visibility,
            wind_speed,
            wind_direction,
            wind_chill: None,
            humidex: None,
            
            // --- POPULATE NEW FIELDS ---
            aqhi_value,
            aqhi_risk,
            sunrise,
            sunset,
        })
    }
    
    // ... (parse_forecasts_from_entries remains unchanged)
    fn parse_forecasts_from_entries(entries: &[roxmltree::Node]) -> Vec<DailyForecast> {
        // ... (function body remains unchanged)
        let mut forecasts = Vec::new();
        let mut current_day: Option<(String, Option<i32>, Option<i32>, String, Option<u32>)> = None;
        
        for entry in entries.iter().take(14) {
            // Title format: "Tuesday: Sunny. High 13." or "Tuesday night: Mainly cloudy. Low 8."
            let title = entry
                .descendants()
                .find(|n| n.has_tag_name("title"))
                .and_then(|n| n.text())
                .unwrap_or("");
            
            let summary = entry
                .descendants()
                .find(|n| n.has_tag_name("summary"))
                .and_then(|n| n.text())
                .unwrap_or("");
            
            let is_night = title.to_lowercase().contains("night");
            
            // Extract day name from title (e.g., "Tuesday" from "Tuesday: Sunny. High 13.")
            let day_name = title.split(':').next().unwrap_or(title).trim().to_string();
            
            // Parse temperature from title
            let high = if title.contains("High") {
                title
                    .split("High")
                    .nth(1)
                    .and_then(|s| s.split('.').next())
                    .and_then(|s| {
                        let trimmed = s.trim();
                        // Handle "plus 4", "minus 5", "zero", or just numbers
                        if trimmed.starts_with("plus") {
                        trimmed.split_whitespace().nth(1).and_then(|n| n.parse().ok())
                        } else if trimmed.starts_with("minus") {
                        trimmed.split_whitespace().nth(1).and_then(|n| n.parse::<i32>().ok()).map(|n| -n)
                        } else if trimmed == "zero" {
                            Some(0)
                        } else {
                            trimmed.parse().ok()
                        }
                    })
            } else {
                None
            };
            
            let low = if title.contains("Low") {
                title
                    .split("Low")
                    .nth(1)
                    .and_then(|s| s.split('.').next())
                    .and_then(|s| {
                        let trimmed = s.trim();
                        // Handle "plus 3" or "minus 5" or just numbers
                        if trimmed.starts_with("plus") {
                            trimmed.split_whitespace().nth(1).and_then(|n| n.parse().ok())
                        } else if trimmed.starts_with("minus") {
                            trimmed.split_whitespace().nth(1).and_then(|n| n.parse::<i32>().ok()).map(|n| -n)
                        } else if trimmed == "zero" {
                            Some(0)
                        } else {
                            trimmed.parse().ok()
                        }
                    })
            } else {
                None
            };
            
            // Parse POP from title (e.g., "POP 30%")
            let pop = if title.contains("POP") {
                title
                    .split("POP")
                    .nth(1)
                    .and_then(|s| s.trim_end_matches('%').trim().parse().ok())
            } else {
                None
            };
            
            // Get condition from title (between : and .)
            let condition = title
                .split(':')
                .nth(1)
                .and_then(|s| s.split('.').next())
                .unwrap_or("")
                .trim()
                .to_string();
            
            if !is_night {
                // Save day forecast
                current_day = Some((day_name, high, None, condition, pop));
            } else if let Some((name, day_high, _, day_condition, day_pop)) = current_day.take() {
                // Combine day and night
                let icon = DailyForecast::get_emoji(&day_condition);
                forecasts.push(DailyForecast {
                    day_name: name,
                    high: day_high,
                    low,
                    summary: day_condition,
                    pop: day_pop.or(pop),
                    icon,
                });
            }
        }
        
        // Add any remaining day forecast
        if let Some((name, high, low, condition, pop)) = current_day {
            let icon = DailyForecast::get_emoji(&condition);
            forecasts.push(DailyForecast {
                day_name: name,
                high,
                low,
                summary: condition,
                pop,
                icon,
            });
        }
        
        forecasts.into_iter().take(7).collect()
    }
}
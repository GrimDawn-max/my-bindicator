use gloo_net::http::Request;
use gloo_console::log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherData {
    pub current: CurrentConditions,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
}

impl WeatherData {
    /// Get forecast for a specific day name (e.g., "Monday", "Tuesday")
    pub fn get_forecast_for_day(&self, day_name: &str) -> Option<&DailyForecast> {
        self.daily.iter().find(|forecast| {
            forecast.day_name.eq_ignore_ascii_case(day_name)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentConditions {
    pub temperature: f32,
    pub condition: String,
    pub icon: String,
    pub humidity: u32,
    pub wind_speed: u32,
    pub wind_direction: String,
    pub pressure: f32,
    pub visibility: f32,
    pub dewpoint: f32,
    pub air_quality: Option<AirQuality>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirQuality {
    pub index: u32,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub time: String,
    pub temperature: i32,
    pub condition: String,
    pub pop: u32,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyForecast {
    pub day_name: String,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub summary: String,
    pub pop: Option<u32>,
    pub icon: String,
}

impl DailyForecast {
    pub fn get_emoji(condition: &str) -> String {
        let condition_lower = condition.to_lowercase();
        if condition_lower.contains("sun") || condition_lower.contains("clear") {
            "☀️".to_string()
        } else if condition_lower.contains("cloud") && condition_lower.contains("sun") {
            "⛅".to_string()
        } else if condition_lower.contains("cloud") {
            "☁️".to_string()
        } else if condition_lower.contains("rain") || condition_lower.contains("shower") {
            "🌧️".to_string()
        } else if condition_lower.contains("snow") {
            "❄️".to_string()
        } else if condition_lower.contains("thunder") || condition_lower.contains("storm") {
            "⛈️".to_string()
        } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
            "🌫️".to_string()
        } else {
            "🌤️".to_string()
        }
    }
}

// Multiple CORS proxy options for reliability
const CORS_PROXIES: &[&str] = &[
    "https://corsproxy.io/?",
    "https://api.allorigins.win/raw?url=",
];

// Toronto RSS feed
const WEATHER_URL: &str = "https://weather.gc.ca/rss/city/on-143_e.xml";

pub async fn fetch_weather_data() -> Result<WeatherData, String> {
    // Try direct fetch first
    log!("Attempting direct fetch from Environment Canada RSS...");
    match try_fetch(WEATHER_URL).await {
        Ok(data) => {
            log!("✓ Direct fetch succeeded");
            return Ok(data);
        }
        Err(e) => {
            let msg = format!("✗ Direct fetch failed: {}. Trying CORS proxies...", e);
            log!(&msg);
        }
    }
    
    // Try each CORS proxy in sequence
    for (i, proxy) in CORS_PROXIES.iter().enumerate() {
        let proxied_url = format!("{}{}", proxy, WEATHER_URL);
        let msg = format!("Attempting proxy {}/{}: {}", i + 1, CORS_PROXIES.len(), *proxy);
        log!(&msg);
        
        match try_fetch(&proxied_url).await {
            Ok(data) => {
                let msg = format!("✓ Success with proxy: {}", *proxy);
                log!(&msg);
                return Ok(data);
            }
            Err(e) => {
                let msg = format!("✗ Proxy {} failed: {}", *proxy, e);
                log!(&msg);
            }
        }
    }
    
    Err("Unable to load weather data from any source. Please check your internet connection.".to_string())
}

async fn try_fetch(url: &str) -> Result<WeatherData, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {:?}", e))?;
    
    if !response.ok() {
        return Err(format!("HTTP {}: {}", response.status(), response.status_text()));
    }
    
    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {:?}", e))?;
    
    parse_rss_xml(&text)
}

fn parse_rss_xml(xml: &str) -> Result<WeatherData, String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    
    let mut current = CurrentConditions {
        temperature: 0.0,
        condition: String::new(),
        icon: String::new(),
        humidity: 0,
        wind_speed: 0,
        wind_direction: String::new(),
        pressure: 0.0,
        visibility: 0.0,
        dewpoint: 0.0,
        air_quality: None,
    };
    
    let mut forecasts = Vec::new();
    let mut buf = Vec::new();
    let mut current_element = String::new();
    let mut in_item = false;
    let mut current_title = String::new();
    let mut current_summary = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if current_element == "entry" {
                    in_item = true;
                    current_title.clear();
                    current_summary.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                let element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if element == "entry" && in_item {
                    in_item = false;
                    // Process the item
                    if current_title.contains("Current Conditions") {
                        // Parse current conditions
                        log!(&format!("🌡️ Parsing current conditions from summary (length: {})", current_summary.len()));
                        parse_current_conditions(&current_title, &current_summary, &mut current);
                    } else if !current_title.is_empty() {
                        // Parse forecast
                        if let Some(forecast) = parse_forecast_item(&current_title, &current_summary) {
                            forecasts.push(forecast);
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !in_item {
                    buf.clear();
                    continue;
                }
                
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                
                match current_element.as_str() {
                    "title" => {
                        current_title = text;
                    }
                    "summary" => {
                        current_summary = text;
                    }
                    _ => {}
                }
            }
            // CRITICAL: Handle CDATA blocks (where current conditions details are stored!)
            Ok(Event::CData(e)) => {
                if !in_item {
                    buf.clear();
                    continue;
                }
                
                // CDATA content doesn't need unescaping - it's already raw
                let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                
                if current_element == "summary" {
                    current_summary = text;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }
    
    log!(&format!("📊 Parsed {} forecast items total", forecasts.len()));
    
    // Separate into hourly and daily
    let (hourly, daily) = separate_forecasts(forecasts);
    
    log!(&format!("📊 Created {} hourly, {} daily forecasts", hourly.len(), daily.len()));
    log!(&format!("🌡️ Current conditions: {}°C, {}% humidity, {} km/h wind", 
        current.temperature, current.humidity, current.wind_speed));
    
    Ok(WeatherData {
        current,
        hourly,
        daily,
    })
}

fn parse_current_conditions(title: &str, summary: &str, current: &mut CurrentConditions) {
    // Title format: "Current Conditions: Mostly Cloudy, 10.4°C"
    if let Some(after_colon) = title.split(':').nth(1) {
        let parts: Vec<&str> = after_colon.split(',').collect();
        if parts.len() >= 2 {
            current.condition = parts[0].trim().to_string();
            current.icon = get_weather_icon(&current.condition);
            
            // Parse temperature from title
            if let Some(temp_str) = parts[1].trim().strip_suffix("°C") {
                current.temperature = temp_str.trim().parse().unwrap_or(0.0);
            }
        }
    }
    
    // Parse HTML summary for detailed conditions
    // Format: <b>Temperature:</b> 10.4&deg;C<br/> <b>Humidity:</b> 83 %<br/> ...
    
    for line in summary.split("<br/>") {
        let line = line.trim();
        
        // Extract value after the bold tag
        if let Some(value_part) = line.split("</b>").nth(1) {
            let value = value_part.trim();
            
            if line.contains("Temperature:") {
                if let Some(temp_str) = value.split("&deg;C").next() {
                    current.temperature = temp_str.trim().parse().unwrap_or(current.temperature);
                }
            } else if line.contains("Humidity:") {
                if let Some(num_str) = value.split('%').next() {
                    current.humidity = num_str.trim().parse().unwrap_or(0);
                }
            } else if line.contains("Pressure") {
                // Format: "99.8 kPa rising"
                if let Some(num_str) = value.split_whitespace().next() {
                    current.pressure = num_str.parse().unwrap_or(0.0);
                }
            } else if line.contains("Visibility:") {
                if let Some(num_str) = value.split_whitespace().next() {
                    current.visibility = num_str.parse().unwrap_or(0.0);
                }
            } else if line.contains("Dewpoint:") {
                if let Some(temp_str) = value.split("&deg;C").next() {
                    current.dewpoint = temp_str.trim().parse().unwrap_or(0.0);
                }
            } else if line.contains("Wind:") {
                // Format: "WSW 17 km/h gust 28 km/h"
                let parts: Vec<&str> = value.split_whitespace().collect();
                if parts.len() >= 3 {
                    current.wind_direction = parts[0].to_string();
                    current.wind_speed = parts[1].parse().unwrap_or(0);
                }
            } else if line.contains("Air Quality Health Index:") {
                if let Some(index_str) = value.split_whitespace().next() {
                    if let Ok(index) = index_str.parse::<u32>() {
                        let category = match index {
                            1..=3 => "Low",
                            4..=6 => "Moderate",
                            7..=10 => "High",
                            _ => "Very High",
                        };
                        current.air_quality = Some(AirQuality {
                            index,
                            category: category.to_string(),
                        });
                    }
                }
            }
        }
    }
}

fn parse_forecast_item(title: &str, summary: &str) -> Option<HourlyForecast> {
    // Skip special items
    if title.contains("SPECIAL WEATHER") || title.contains("Notice") {
        return None;
    }
    
    let parts: Vec<&str> = title.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let period = parts[0].trim().to_string();
    let condition_part = parts[1].trim();
    
    // Extract condition (first sentence before period)
    let condition = condition_part
        .split('.')
        .next()
        .unwrap_or(condition_part)
        .trim()
        .to_string();
    
    // Extract temperature
    let temp = if let Some(high) = extract_number(title, "High") {
        high as i32
    } else if let Some(low) = extract_number(title, "Low") {
        low as i32
    } else {
        0
    };
    
    // Extract POP from title or summary
    let pop = extract_pop(title).max(extract_pop(summary));
    
    Some(HourlyForecast {
        time: period,
        temperature: temp,
        condition: condition.clone(),
        pop,
        icon: get_weather_icon(&condition),
    })
}

fn separate_forecasts(forecasts: Vec<HourlyForecast>) -> (Vec<HourlyForecast>, Vec<DailyForecast>) {
    let mut hourly = Vec::new();
    let mut daily = Vec::new();
    let mut current_day: Option<(String, Option<i32>, Option<i32>, String, Option<u32>)> = None;
    
    for forecast in forecasts {
        let period_name = &forecast.time;
        let is_night = period_name.to_lowercase().contains("night");
        
        // Extract day name
        let day_name = period_name
            .split_whitespace()
            .next()
            .unwrap_or(period_name)
            .to_string();
        
        // Add to hourly (all forecasts)
        hourly.push(forecast.clone());
        
        // Build daily forecasts
        let temp = Some(forecast.temperature);
        let pop = if forecast.pop > 0 { Some(forecast.pop) } else { None };
        
        if !is_night {
            // Day forecast - save as high
            current_day = Some((day_name, temp, None, forecast.condition.clone(), pop));
        } else {
            // Night forecast
            if let Some((name, day_high, _, day_condition, day_pop)) = current_day.take() {
                // Combine day and night
                let icon = DailyForecast::get_emoji(&day_condition);
                daily.push(DailyForecast {
                    day_name: name,
                    high: day_high,
                    low: temp,
                    summary: day_condition,
                    pop: day_pop.or(pop),
                    icon,
                });
            } else {
                // Night only
                let icon = DailyForecast::get_emoji(&forecast.condition);
                daily.push(DailyForecast {
                    day_name,
                    high: None,
                    low: temp,
                    summary: forecast.condition.clone(),
                    pop,
                    icon,
                });
            }
        }
    }
    
    // Add remaining day forecast if any
    if let Some((name, high, low, condition, pop)) = current_day {
        let icon = DailyForecast::get_emoji(&condition);
        daily.push(DailyForecast {
            day_name: name,
            high,
            low,
            summary: condition,
            pop,
            icon,
        });
    }
    
    (hourly, daily.into_iter().take(7).collect())
}

fn extract_number(text: &str, keyword: &str) -> Option<f32> {
    let text_lower = text.to_lowercase();
    let keyword_lower = keyword.to_lowercase();
    
    if let Some(pos) = text_lower.find(&keyword_lower) {
        let after = &text[pos + keyword.len()..];
        // Handle "plus" or "minus" modifiers
        let words: Vec<&str> = after.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase() == "plus" || word.to_lowercase() == "minus" {
                // Next word should be the number
                if i + 1 < words.len() {
                    if let Ok(num) = words[i + 1].trim_matches(|c: char| !c.is_numeric() && c != '.').parse::<f32>() {
                        return Some(if *word == "minus" { -num } else { num });
                    }
                }
            } else {
                let cleaned = word.trim_matches(|c: char| !c.is_numeric() && c != '-' && c != '.');
                if let Ok(num) = cleaned.parse::<f32>() {
                    return Some(num);
                }
            }
        }
    }
    None
}

fn extract_pop(text: &str) -> u32 {
    // Look for "POP XX%"
    if let Some(pos) = text.find("POP") {
        let after = &text[pos + 3..];
        for word in after.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_numeric());
            if let Ok(num) = cleaned.parse::<u32>() {
                return num;
            }
        }
    }
    0
}

fn get_weather_icon(condition: &str) -> String {
    let condition_lower = condition.to_lowercase();
    if condition_lower.contains("sun") || condition_lower.contains("clear") {
        "☀️".to_string()
    } else if condition_lower.contains("cloud") && (condition_lower.contains("sun") || condition_lower.contains("clear")) {
        "⛅".to_string()
    } else if condition_lower.contains("cloud") {
        "☁️".to_string()
    } else if condition_lower.contains("rain") || condition_lower.contains("shower") || condition_lower.contains("drizzle") {
        "🌧️".to_string()
    } else if condition_lower.contains("snow") || condition_lower.contains("flurr") {
        "❄️".to_string()
    } else if condition_lower.contains("thunder") || condition_lower.contains("storm") {
        "⛈️".to_string()
    } else if condition_lower.contains("fog") || condition_lower.contains("mist") {
        "🌫️".to_string()
    } else {
        "🌤️".to_string()
    }
}

use gloo_net::http::Request;
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use futures::future::{select, Either};
use serde::{Deserialize, Serialize};

// Timeout for fetch in seconds
const FETCH_TIMEOUT_SECS: u32 = 10;

// Environment Canada GeoMet API - free, no auth, CORS enabled
const WEATHER_API_URL: &str = "https://api.weather.gc.ca/collections/citypageweather-realtime/items?f=json&identifier=on-143";
const AQHI_API_URL: &str = "https://api.weather.gc.ca/collections/aqhi-observations-realtime/items?f=json&location_id=FCWYG&sortby=-observation_datetime&limit=1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherData {
    pub current: CurrentConditions,
    pub hourly: Vec<HourlyForecast>,
    pub daily: Vec<DailyForecast>,
    pub warnings: Vec<WeatherWarning>,
    pub sun: Option<SunTimes>,
}

impl WeatherData {
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
    pub wind_gust: Option<u32>,
    pub wind_chill: Option<i32>,
    pub pressure: f32,
    pub pressure_tendency: Option<String>,
    pub dewpoint: f32,
    pub visibility: Option<f32>,
    pub station: String,
    pub air_quality: Option<AirQuality>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirQuality {
    pub index: f32,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherWarning {
    pub description: String,
    pub alert_level: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SunTimes {
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub time: String,
    pub temperature: i32,
    pub condition: String,
    pub pop: u32,
    pub icon: String,
    pub wind_speed: u32,
    pub wind_direction: String,
    pub wind_chill: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyForecast {
    pub day_name: String,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub summary: String,
    pub pop: Option<u32>,
    pub icon: String,
    pub uv_index: Option<String>,
    pub wind_chill: Option<String>,
    pub wind_summary: Option<String>,
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
}

pub async fn fetch_weather_data() -> Result<WeatherData, String> {
    log!("Fetching weather from Environment Canada GeoMet API...");

    // Race the fetch against a timeout
    let fetch_future = Box::pin(fetch_and_parse());
    let timeout_future = Box::pin(TimeoutFuture::new(FETCH_TIMEOUT_SECS * 1000));

    match select(fetch_future, timeout_future).await {
        Either::Left((result, _)) => result,
        Either::Right((_, _)) => Err(format!("Request timed out after {} seconds", FETCH_TIMEOUT_SECS)),
    }
}

async fn fetch_and_parse() -> Result<WeatherData, String> {
    // Fetch main weather data
    let response = Request::get(WEATHER_API_URL)
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

    let mut weather_data = parse_api_response(&text)?;

    // Try to fetch AQHI data (don't fail if unavailable)
    if let Ok(aqhi) = fetch_aqhi().await {
        weather_data.current.air_quality = Some(aqhi);
    }

    Ok(weather_data)
}

async fn fetch_aqhi() -> Result<AirQuality, String> {
    let response = Request::get(AQHI_API_URL)
        .send()
        .await
        .map_err(|e| format!("AQHI network error: {:?}", e))?;

    if !response.ok() {
        return Err("AQHI fetch failed".to_string());
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("AQHI read error: {:?}", e))?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("AQHI JSON error: {:?}", e))?;

    let aqhi_value = json.get("features")
        .and_then(|f| f.as_array())
        .and_then(|arr| arr.first())
        .and_then(|f| f.get("properties"))
        .and_then(|p| p.get("aqhi"))
        .and_then(|v| v.as_f64())
        .ok_or("No AQHI value found")?;

    let index = aqhi_value as f32;
    let category = match index.round() as u32 {
        1..=3 => "Low Risk",
        4..=6 => "Moderate Risk",
        7..=10 => "High Risk",
        _ => "Very High Risk",
    }.to_string();

    Ok(AirQuality { index, category })
}

fn parse_api_response(json_str: &str) -> Result<WeatherData, String> {
    let json: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {:?}", e))?;

    let features = json.get("features")
        .and_then(|f| f.as_array())
        .ok_or("No features array in response")?;

    let props = features.first()
        .and_then(|f| f.get("properties"))
        .ok_or("No properties in feature")?;

    // Parse current conditions
    let current = parse_current_conditions(props)?;

    // Parse forecasts
    let (hourly, daily) = parse_forecasts(props);

    // Parse warnings
    let warnings = parse_warnings(props);

    // Parse sunrise/sunset
    let sun = parse_sun_times(props);

    log!(&format!("✓ Weather loaded: {}°C, {}", current.temperature, current.condition));

    Ok(WeatherData {
        current,
        hourly,
        daily,
        warnings,
        sun,
    })
}

fn parse_current_conditions(props: &serde_json::Value) -> Result<CurrentConditions, String> {
    let cc = props.get("currentConditions")
        .ok_or("No currentConditions in response")?;

    let temperature = cc.get("temperature")
        .and_then(|t| t.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let condition = cc.get("condition")
        .and_then(|c| c.get("en"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();

    let humidity = cc.get("relativeHumidity")
        .and_then(|h| h.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let wind_speed = cc.get("wind")
        .and_then(|w| w.get("speed"))
        .and_then(|s| s.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let wind_direction = cc.get("wind")
        .and_then(|w| w.get("direction"))
        .and_then(|d| d.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let wind_gust = cc.get("wind")
        .and_then(|w| w.get("gust"))
        .and_then(|g| g.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .filter(|&v| v > 0);

    let wind_chill = cc.get("windChill")
        .and_then(|w| w.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    let pressure = cc.get("pressure")
        .and_then(|p| p.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let pressure_tendency = cc.get("pressure")
        .and_then(|p| p.get("tendency"))
        .and_then(|t| t.get("en"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if let Some(ref t) = pressure_tendency {
        log!(&format!("Pressure tendency from API: '{}'", t));
    }

    let dewpoint = cc.get("dewpoint")
        .and_then(|d| d.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let visibility = cc.get("visibility")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let station = cc.get("station")
        .and_then(|s| s.get("value"))
        .and_then(|v| v.get("en"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let icon = get_weather_icon(&condition);

    Ok(CurrentConditions {
        temperature,
        condition,
        icon,
        humidity,
        wind_speed,
        wind_direction,
        wind_gust,
        wind_chill,
        pressure,
        pressure_tendency,
        dewpoint,
        visibility,
        station,
        air_quality: None,
    })
}

fn parse_warnings(props: &serde_json::Value) -> Vec<WeatherWarning> {
    let mut warnings = Vec::new();

    if let Some(warning_array) = props.get("warnings").and_then(|w| w.as_array()) {
        for w in warning_array {
            let description = w.get("description")
                .and_then(|d| d.get("en"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let alert_level = w.get("alertColourLevel")
                .and_then(|a| a.get("en"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let url = w.get("url")
                .and_then(|u| u.get("en"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !description.is_empty() {
                warnings.push(WeatherWarning {
                    description,
                    alert_level,
                    url,
                });
            }
        }
    }

    warnings
}

fn parse_sun_times(props: &serde_json::Value) -> Option<SunTimes> {
    let rise_set = props.get("riseSet")?;

    let sunrise_utc = rise_set.get("sunrise")
        .and_then(|s| s.get("en"))
        .and_then(|v| v.as_str())?;

    let sunset_utc = rise_set.get("sunset")
        .and_then(|s| s.get("en"))
        .and_then(|v| v.as_str())?;

    // Convert UTC to local time display (just extract time part for now)
    let sunrise = format_utc_to_local_time(sunrise_utc);
    let sunset = format_utc_to_local_time(sunset_utc);

    Some(SunTimes { sunrise, sunset })
}

fn format_utc_to_local_time(utc_str: &str) -> String {
    // UTC format: "2026-01-30T12:37:00Z"
    // Toronto is UTC-5, so subtract 5 hours
    if let Some(time_part) = utc_str.split('T').nth(1) {
        if let Some(hour_str) = time_part.split(':').next() {
            if let Ok(hour) = hour_str.parse::<i32>() {
                let local_hour = (hour - 5 + 24) % 24;
                let minute = time_part.split(':').nth(1).unwrap_or("00");
                let am_pm = if local_hour < 12 { "AM" } else { "PM" };
                let display_hour = if local_hour == 0 { 12 } else if local_hour > 12 { local_hour - 12 } else { local_hour };
                return format!("{}:{} {}", display_hour, minute, am_pm);
            }
        }
    }
    utc_str.to_string()
}

fn parse_forecasts(props: &serde_json::Value) -> (Vec<HourlyForecast>, Vec<DailyForecast>) {
    let mut hourly = Vec::new();
    let mut daily = Vec::new();

    // Parse hourly forecasts
    if let Some(hfg) = props.get("hourlyForecastGroup") {
        if let Some(forecasts) = hfg.get("hourlyForecasts").and_then(|f| f.as_array()) {
            for fc in forecasts.iter().take(24) {
                let timestamp = fc.get("timestamp")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                // Extract hour from timestamp for display
                let time = format_utc_to_local_time(timestamp);

                let temperature = fc.get("temperature")
                    .and_then(|t| t.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                let condition = fc.get("condition")
                    .and_then(|c| c.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let pop = fc.get("lop")
                    .and_then(|l| l.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let wind_speed = fc.get("wind")
                    .and_then(|w| w.get("speed"))
                    .and_then(|s| s.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                let wind_direction = fc.get("wind")
                    .and_then(|w| w.get("direction"))
                    .and_then(|d| d.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let wind_chill = fc.get("windChill")
                    .and_then(|w| w.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let icon = DailyForecast::get_emoji(&condition);

                hourly.push(HourlyForecast {
                    time,
                    temperature,
                    condition,
                    pop,
                    icon,
                    wind_speed,
                    wind_direction,
                    wind_chill,
                });
            }
        }
    }

    // Parse daily forecasts from forecastGroup
    if let Some(fg) = props.get("forecastGroup") {
        if let Some(forecasts) = fg.get("forecasts").and_then(|f| f.as_array()) {
            let mut day_forecasts: std::collections::HashMap<String, (Option<i32>, Option<i32>, String, Option<u32>, Option<String>, Option<String>, Option<String>)> = std::collections::HashMap::new();

            for fc in forecasts {
                let period = fc.get("period")
                    .and_then(|p| p.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let summary = fc.get("abbreviatedForecast")
                    .and_then(|a| a.get("textSummary"))
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Get temperature
                let temp_info = fc.get("temperatures")
                    .and_then(|t| t.get("temperature"))
                    .and_then(|t| t.as_array())
                    .and_then(|arr| arr.first());

                let temp = temp_info
                    .and_then(|t| t.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let temp_class = temp_info
                    .and_then(|t| t.get("class"))
                    .and_then(|c| c.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Extract POP from text
                let text_summary = fc.get("textSummary")
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let pop = extract_pop(text_summary);

                // UV index
                let uv_index = fc.get("uv")
                    .and_then(|u| u.get("textSummary"))
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Wind chill
                let wind_chill = fc.get("windChill")
                    .and_then(|w| w.get("textSummary"))
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Wind summary
                let wind_summary = fc.get("winds")
                    .and_then(|w| w.get("textSummary"))
                    .and_then(|t| t.get("en"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Build daily forecasts by combining day/night
                let is_night = period.to_lowercase().contains("night") ||
                               period.to_lowercase().contains("tonight");

                let day_name = if is_night {
                    period.replace(" night", "").replace("tonight", "Friday")
                } else {
                    period.to_string()
                };

                let entry = day_forecasts.entry(day_name.clone()).or_insert((None, None, String::new(), None, None, None, None));

                if temp_class == "high" || !is_night {
                    entry.0 = temp; // high
                    if entry.2.is_empty() {
                        entry.2 = summary;
                    }
                    // Prefer daytime UV/wind info
                    if uv_index.is_some() {
                        entry.4 = uv_index;
                    }
                    if wind_summary.is_some() {
                        entry.6 = wind_summary;
                    }
                } else {
                    entry.1 = temp; // low
                }
                if pop > 0 && entry.3.unwrap_or(0) < pop {
                    entry.3 = Some(pop);
                }
                // Wind chill from either day or night
                if wind_chill.is_some() && entry.5.is_none() {
                    entry.5 = wind_chill;
                }
            }

            // Convert to daily forecasts (preserve order)
            let mut seen_days = std::collections::HashSet::new();
            for fc in forecasts {
                let period = fc.get("period")
                    .and_then(|p| p.get("value"))
                    .and_then(|v| v.get("en"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let is_night = period.to_lowercase().contains("night") ||
                               period.to_lowercase().contains("tonight");
                let day_name = if is_night {
                    period.replace(" night", "").replace("tonight", "Friday")
                } else {
                    period.to_string()
                };

                if seen_days.contains(&day_name) {
                    continue;
                }
                seen_days.insert(day_name.clone());

                if let Some((high, low, summary, pop, uv_index, wind_chill, wind_summary)) = day_forecasts.get(&day_name) {
                    let icon = DailyForecast::get_emoji(summary);
                    daily.push(DailyForecast {
                        day_name,
                        high: *high,
                        low: *low,
                        summary: summary.clone(),
                        pop: *pop,
                        icon,
                        uv_index: uv_index.clone(),
                        wind_chill: wind_chill.clone(),
                        wind_summary: wind_summary.clone(),
                    });
                }
            }
        }
    }

    // Limit to 7 days
    daily.truncate(7);

    (hourly, daily)
}

fn extract_pop(text: &str) -> u32 {
    let text_lower = text.to_lowercase();
    if let Some(pos) = text_lower.find("percent") {
        let before = &text[..pos];
        for word in before.split_whitespace().rev() {
            if let Ok(num) = word.parse::<u32>() {
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

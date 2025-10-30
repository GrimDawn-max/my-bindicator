use crate::{
    components::{
        weather_daily::{DailyComponent, DailyComponentProps},
        weather_hourly::HourlyComponent,
    },
    context::weather::WeatherContext,
};
use chrono::DateTime;
use yew::prelude::*;
use gloo_console::log;

#[function_component]
pub fn WeatherComponent() -> Html {
    let weather_ctx = use_context::<WeatherContext>().unwrap();
    
    if !weather_ctx.is_loaded {
        return html! {
            <div class="text-white">
                <p>{"Loading weather data..."}</p>
            </div>
        };
    }
    
    let weather = weather_ctx.weather.clone();
    
    if weather.hourly.time.is_empty() || weather.daily.time.is_empty() {
        return html! {
            <div class="text-white">
                <p>{"No weather data available"}</p>
            </div>
        };
    }
    
    let offset_sec = weather.utc_offset_seconds / 60 / 60;
    let offset_hours = if offset_sec >= 0 {
        format!("+{:02}:00", offset_sec)
    } else {
        format!("{:03}:00", offset_sec)
    };
    
    log!(format!("Weather daily time array: {:?}", weather.daily.time));
    
    // Create a vec of exactly 7 unique days
    let mut daily_cards = Vec::new();
    for i in 0..weather.daily.time.len().min(7) {
        let time = &weather.daily.time[i];
        let temp_max = weather.daily.temperature_2m_max[i];
        let temp_min = weather.daily.temperature_2m_min[i];
        let precipitation = weather.daily.precipitation_sum[i];
        let precipitation_probability_max = weather.daily.precipitation_probability_max[i];
        let code = weather.daily.weather_code[i];
        
        // Parse dates at noon to avoid DST transition issues
        let date = DateTime::parse_from_rfc3339(&format!("{time}T12:00:00{offset_hours}"));
        let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunrise[i]));
        let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunset[i]));
        
        if date.is_ok() && sunrise.is_ok() && sunset.is_ok() {
            log!(format!("Adding card {}: {} - {}", i, time, date.as_ref().unwrap().format("%a")));
            daily_cards.push((
                time.clone(),
                DailyComponentProps {
                    weather_code: code,
                    temp_max,
                    temp_min,
                    precipitation_sum: precipitation,
                    precipitation_probability_max,
                    date: date.unwrap().into(),
                    sunrise: sunrise.unwrap().into(),
                    sunset: sunset.unwrap().into(),
                },
            ));
        }
    }
    
    log!(format!("Total cards to render: {}", daily_cards.len()));
    
    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-white mt-3">
            {
                daily_cards.into_iter().map(|(key, props)| {
                    html!{ <DailyComponent key={key} ..props /> }
                }).collect::<Html>()
            }
            </div>
        </>
    }
}
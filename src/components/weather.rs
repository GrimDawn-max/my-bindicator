use crate::{
    components::{
        weather_daily::{DailyComponent, DailyComponentProps},
        weather_hourly::HourlyComponent,
    },
    context::weather::WeatherContext,
};
use chrono::DateTime;
use yew::prelude::*;

#[function_component]
pub fn WeatherComponent() -> Html {
    let weather_ctx = use_context::<WeatherContext>().unwrap();
    
    // Don't render until data is loaded
    if !weather_ctx.is_loaded {
        return html! {
            <div class="text-white">
                <p>{"Loading weather data..."}</p>
            </div>
        };
    }
    
    let weather = weather_ctx.weather.clone();
    
    // Additional safety check - make sure we have data
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
    
    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-white mt-3">
            {
                weather.daily.time.clone().iter().enumerate().map(|(i, time)| {
                    let temp_max = weather.daily.temperature_2m_max.clone()[i];
                    let temp_min = weather.daily.temperature_2m_min.clone()[i];
                    let precipitation = weather.daily.precipitation_sum.clone()[i];
                    let precipitation_probability_max = weather.daily.precipitation_probability_max.clone()[i];
                    let code = weather.daily.weather_code.clone()[i];
                    let date = DateTime::parse_from_rfc3339(&format!("{time}T00:00:00{offset_hours}"));
                    let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunrise.clone()[i]));
                    let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunset.clone()[i]));
                    if date.is_ok() {
                        let props = DailyComponentProps {
                            weather_code: code.to_owned(),
                            temp_max: temp_max.to_owned(),
                            temp_min: temp_min.to_owned(),
                            precipitation_sum: precipitation.to_owned(),
                            precipitation_probability_max: precipitation_probability_max.to_owned(),
                            date: date.unwrap().to_owned().into(),
                            sunrise: sunrise.unwrap().to_owned().into(),
                            sunset: sunset.unwrap().to_owned().into(),
                        };
                        html!{
                            <DailyComponent ..props.clone() />
                        }
                    } else {
                        html!{}
                    }
                }).collect::<Html>()
            }
            </div>
        </>
    }
}
// src/weather/components.rs - COMPLETE REPLACEMENT

use yew::prelude::*;
use gloo_console::log;
use crate::weather::api::WeatherData;

#[derive(Properties, PartialEq)]
pub struct WeatherDisplayProps {
    pub weather: WeatherData,
}

#[function_component(WeatherDisplay)]
pub fn weather_display(props: &WeatherDisplayProps) -> Html {
    let weather = &props.weather;
    
    html! {
        <div class="weather-display">
            {render_current(&weather.current)}
            {render_daily_forecast(&weather.daily)}
        </div>
    }
}

fn render_current(current: &crate::weather::api::CurrentConditions) -> Html {
    html! {
        <div class="card mb-3 current-weather">
            <div class="card-body">
                <h5 class="card-title">{"Current Conditions"}</h5>
                <div class="row">
                    <div class="col-md-6">
                        <div class="d-flex align-items-center mb-2">
                            <span class="weather-icon me-2" style="font-size: 3rem;">{&current.icon}</span>
                            <div>
                                <h2 class="mb-0">{format!("{}°C", current.temperature)}</h2>
                                <p class="mb-0">{&current.condition}</p>
                            </div>
                        </div>
                    </div>
                    <div class="col-md-6">
                        <div class="weather-details small">
                            <div class="d-flex justify-content-between mb-1">
                                <span>{"Humidity:"}</span>
                                <strong>{format!("{}%", current.humidity)}</strong>
                            </div>
                            <div class="d-flex justify-content-between mb-1">
                                <span>{"Wind:"}</span>
                                <strong>{format!("{} km/h {}", current.wind_speed, current.wind_direction)}</strong>
                            </div>
                            <div class="d-flex justify-content-between mb-1">
                                <span>{"Pressure:"}</span>
                                <strong>{format!("{:.1} kPa", current.pressure)}</strong>
                            </div>
                            <div class="d-flex justify-content-between mb-1">
                                <span>{"Visibility:"}</span>
                                <strong>{format!("{:.1} km", current.visibility)}</strong>
                            </div>
                            <div class="d-flex justify-content-between">
                                <span>{"Dewpoint:"}</span>
                                <strong>{format!("{:.1}°C", current.dewpoint)}</strong>
                            </div>
                            if let Some(ref aq) = current.air_quality {
                                <div class="d-flex justify-content-between mt-1">
                                    <span>{"Air Quality:"}</span>
                                    <strong>{format!("{} ({})", aq.index, aq.category)}</strong>
                                </div>
                            }
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn render_daily_forecast(forecasts: &[crate::weather::api::DailyForecast]) -> Html {
    html! {
        <div class="row g-2 mb-3">
            <div class="col-12">
                <h5>{"7-Day Forecast"}</h5>
            </div>
            {
                forecasts.iter().map(|forecast| {
                    let high_display = forecast.high
                        .map(|h| format!("{}", h))
                        .unwrap_or_else(|| "N/A".to_string());
                    
                    let low_display = forecast.low
                        .map(|l| format!("{}", l))
                        .unwrap_or_else(|| "N/A".to_string());
                    
                    let pop_display = forecast.pop
                        .map(|p| format!("{}%", p))
                        .unwrap_or_else(|| "N/A".to_string());
                    
                    html! {
                        <div class="col" key={forecast.day_name.clone()}>
                            <div class="card">
                                <div class="card-header text-center p-0 text-body">
                                    {&forecast.day_name}
                                </div>
                                <div class="card-body d-flex flex-column align-items-center gap-1 p-0">
                                    <div class="display-3">
                                        {&forecast.icon}
                                    </div>
                                    <div class="text-nowrap text-body fw-bold fs-5">
                                        {format!("{} - {} ºC", high_display, low_display)}
                                    </div>
                                    <div class="text-nowrap text-body fw-bold">
                                        {&forecast.summary}
                                    </div>
                                    <div class="text-body fw-bold">
                                        {format!("POP {}", pop_display)}
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

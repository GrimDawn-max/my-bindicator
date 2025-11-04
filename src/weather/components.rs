// src/weather/components.rs

use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use gloo_timers::callback::Interval;
use crate::weather::{EnvironmentCanadaClient, WeatherData};

pub enum WeatherMsg {
    LoadWeather,
    WeatherLoaded(Result<WeatherData, String>),
}

#[derive(Properties, PartialEq)]
pub struct WeatherProps {
    #[prop_or_default]
    pub on_weather_loaded: Option<Callback<WeatherData>>,
}

pub struct WeatherDisplay {
    weather: Option<WeatherData>,
    loading: bool,
    error: Option<String>,
    _interval: Option<Interval>,
}

impl Component for WeatherDisplay {
    type Message = WeatherMsg;
    type Properties = WeatherProps;
    
    fn create(ctx: &Context<Self>) -> Self {
        // Load weather on mount
        ctx.link().send_message(WeatherMsg::LoadWeather);
        
        // Auto-refresh every 15 minutes
        let link = ctx.link().clone();
        let interval = Interval::new(15 * 60 * 1000, move || {
            link.send_message(WeatherMsg::LoadWeather);
        });
        
        Self {
            weather: None,
            loading: true,
            error: None,
            _interval: Some(interval),
        }
    }
    
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            WeatherMsg::LoadWeather => {
                self.loading = true;
                let link = ctx.link().clone();
                
              spawn_local(async move {
                let client = EnvironmentCanadaClient::toronto();
                let result = client.fetch_weather().await;
                link.send_message(WeatherMsg::WeatherLoaded(result));
              });
                
                true
            }
            WeatherMsg::WeatherLoaded(result) => {
                self.loading = false;
                
                match result {
                    Ok(data) => {
                        // Notify parent component
                        if let Some(callback) = &ctx.props().on_weather_loaded {
                            callback.emit(data.clone());
                        }
                        self.weather = Some(data);
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
                
                true
            }
        }
    }
    
    fn view(&self, _ctx: &Context<Self>) -> Html {
        if self.loading && self.weather.is_none() {
            return html! {
                <div class="loading">{"Loading weather..."}</div>
            };
        }
        
        if let Some(ref weather) = self.weather {
            self.render_weather(weather)
        } else if let Some(ref error) = self.error {
            html! {
                <div class="error">
                    <p>{"Unable to load weather data"}</p>
                    <small>{error}</small>
                </div>
            }
        } else {
            html! { <div>{"No data"}</div> }
        }
    }
}

impl WeatherDisplay {
    fn render_weather(&self, weather: &WeatherData) -> Html {
        html! {
            <>
                {self.render_warnings(&weather.warnings)}
                {self.render_current(&weather.current)}
                {self.render_forecast(&weather.forecasts)}
            </>
        }
    }
    
    fn render_warnings(&self, warnings: &[crate::weather::models::WeatherWarning]) -> Html {
        if warnings.is_empty() {
            return html! {};
        }
        
        let warning = &warnings[0];
        let class = if warning.priority == "high" {
            "weather-alert severe"
        } else {
            "weather-alert"
        };
        
        html! {
            <div class={class}>
                <div class="alert-icon">{"⚠️"}</div>
                <div class="alert-content">
                    <h3>{&warning.warning_type}</h3>
                    <p>{&warning.description}</p>
                </div>
            </div>
        }
    }
    
    fn render_current(&self, current: &crate::weather::models::CurrentConditions) -> Html {
        html! {
            <div class="card weather-card">
                <h2 class="section-title">
                    <span>{"🌤️"}</span>
                    <span>{"Current Weather"}</span>
                    <span class="ec-badge">{"🍁"}</span>
                </h2>
                
                <div class="current-weather">
                    <div class="temp-section">
                        <div class="temp-display">{format!("{}°C", current.temperature.round())}</div>
                        <div class="condition-text">{&current.condition}</div>
                    </div>
                    
                    <div class="weather-details">
                        <div class="detail-item">
                            <div class="detail-label">{"💧 Humidity"}</div>
                            <div class="detail-value">
                                {current.humidity.map(|h| format!("{}%", h)).unwrap_or_else(|| "--".to_string())}
                            </div>
                        </div>
                        
                        <div class="detail-item">
                            <div class="detail-label">{"💨 Wind"}</div>
                            <div class="detail-value">{current.wind_description()}</div>
                        </div>
                        
                        <div class="detail-item">
                            <div class="detail-label">{"🌡️ Feels Like"}</div>
                            <div class="detail-value">{format!("{}°C", current.feels_like().round())}</div>
                        </div>
                        
                        <div class="detail-item">
                            <div class="detail-label">{"👁️ Visibility"}</div>
                            <div class="detail-value">
                                {current.visibility.map(|v| format!("{} km", v)).unwrap_or_else(|| "--".to_string())}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
    
    fn render_forecast(&self, forecasts: &[crate::weather::models::DailyForecast]) -> Html {
        if forecasts.is_empty() {
            return html! {};
        }
        
        html! {
            <div class="card forecast-card">
                <h2 class="section-title">
                    <span>{"📅"}</span>
                    <span>{"7-Day Forecast"}</span>
                </h2>
                
                <div class="forecast-grid">
                    {for forecasts.iter().enumerate().map(|(i, forecast)| {
                        let is_today = i == 0;
                        let class = if is_today { "forecast-day today" } else { "forecast-day" };
                        
                        html! {
                            <div class={class}>
                                <div class="day-name">{&forecast.day_name}</div>
                                <div class="weather-icon">{&forecast.icon}</div>
                                <div class="forecast-temps">
                                    {if let Some(high) = forecast.high {
                                        html! { <div class="temp-high">{format!("{}°", high)}</div> }
                                    } else {
                                        html! {}
                                    }}
                                    {if let Some(low) = forecast.low {
                                        html! { <div class="temp-low">{format!("{}°", low)}</div> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                                {if let Some(pop) = forecast.pop {
                                    html! { <div class="pop">{format!("💧 {}%", pop)}</div> }
                                } else {
                                    html! {}
                                }}
                                <div class="forecast-summary">{&forecast.summary}</div>
                            </div>
                        }
                    })}
                </div>
            </div>
        }
    }
}
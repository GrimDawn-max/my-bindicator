// src/components/weather.rs
use crate::{
    components::weather_daily::DailyComponent,  // Removed DailyComponentProps - not needed
    context::weather::WeatherContext,
};
use yew::prelude::*;
use gloo_console::log;

#[function_component]
pub fn WeatherComponent() -> Html {
    let weather_ctx = use_context::<WeatherContext>().unwrap();
    
    if !weather_ctx.is_loaded {
        return html! {
            <div class="text-body">
                <p>{"Loading weather data..."}</p>
            </div>
        };
    }
    
    let weather = weather_ctx.weather.clone();
    
    if weather.forecasts.is_empty() {
        return html! {
            <div class="text-body">
                <p>{"No weather data available"}</p>
            </div>
        };
    }
    
    let daily_cards = weather.forecasts.iter().map(|forecast| {
        html! {
            <DailyComponent 
                key={forecast.day_name.clone()}
                day_name={forecast.day_name.clone()}
                icon={forecast.icon.clone()}
                summary={forecast.summary.clone()}
                high={forecast.high}
                low={forecast.low}
                pop={forecast.pop}
            />
        }
    }).collect::<Html>();
    
    log!(format!("Total cards to render: {}", weather.forecasts.len()));
    
    html! {
        <>
            // Current Weather Info (The current temperature display component usually sits here)
            // Assuming your CurrentComponent is implicitly rendered elsewhere or will be added.
            
            // The daily cards
            <div class="card-group text-body mt-3">
                { daily_cards }
            </div>
        </>
    }
}
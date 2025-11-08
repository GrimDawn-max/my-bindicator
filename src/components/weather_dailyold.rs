// src/components/weather_daily.rs
use yew::{function_component, html, Html, Properties};
use crate::weather::api::DailyForecast;

// Individual daily card component
#[allow(dead_code)] // Used by Yew macro
#[derive(Clone, PartialEq, Properties)]
pub struct DailyComponentProps {
    pub day_name: String,
    pub icon: String,
    pub summary: String,
    pub high: Option<i32>,
    pub low: Option<i32>,
    pub pop: Option<u32>,
}

#[function_component]
pub fn DailyComponent(props: &DailyComponentProps) -> Html {
    // Format temperatures, handling None values
    let high_display = props.high
        .map(|h| format!("{}", h))
        .unwrap_or_else(|| "N/A".to_string());
    
    let low_display = props.low
        .map(|l| format!("{}", l))
        .unwrap_or_else(|| "N/A".to_string());
    
    // Format precipitation probability, handling None values
    let pop_display = props.pop
        .map(|p| format!("{}%", p))
        .unwrap_or_else(|| "N/A".to_string());
    
    html! {
        <div class="card">
            <div class="card-header text-center p-0 text-body">
                { &props.day_name }
            </div>
            <div class="card-body d-flex flex-column align-items-center gap-1 p-0">
                // Render the emoji icon
                <div class="display-3">
                    { &props.icon }
                </div>
                
                <div class="text-nowrap text-body fw-bold fs-5">
                    {format!("{} - {} ºC", high_display, low_display)}
                </div>
                
                <div class="text-nowrap text-body fw-bold">
                    { &props.summary }
                </div>
                
                <div class="text-body fw-bold">
                    {format!("POP {}", pop_display)}
                </div>
            </div>
        </div>
    }
}

// Wrapper component that renders all daily forecasts
#[derive(Clone, PartialEq, Properties)]
pub struct WeatherDailyProps {
    pub forecasts: Vec<DailyForecast>,
}

#[function_component(WeatherDaily)]
pub fn weather_daily(props: &WeatherDailyProps) -> Html {
    html! {
        <div class="row g-2 mb-3">
            <div class="col-12">
                <h5>{"7-Day Forecast"}</h5>
            </div>
            {
                props.forecasts.iter().map(|forecast| {
                    html! {
                        <div class="col" key={forecast.day_name.clone()}>
                            <DailyComponent
                                day_name={forecast.day_name.clone()}
                                icon={forecast.icon.clone()}
                                summary={forecast.summary.clone()}
                                high={forecast.high}
                                low={forecast.low}
                                pop={forecast.pop}
                            />
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

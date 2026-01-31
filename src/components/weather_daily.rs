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
    pub uv_index: Option<String>,
    pub wind_chill: Option<String>,
    pub wind_summary: Option<String>,
}

#[function_component]
pub fn DailyComponent(props: &DailyComponentProps) -> Html {
    // Format temperature display based on what's available
    let temp_display = match (props.high, props.low) {
        (Some(h), Some(l)) => format!("{}° / {}°C", h, l),
        (Some(h), None) => format!("High {}°C", h),
        (None, Some(l)) => format!("Low {}°C", l),
        (None, None) => "N/A".to_string(),
    };

    // Use placeholder if summary is empty
    let summary_display = if props.summary.is_empty() {
        "\u{00A0}".to_string() // non-breaking space as placeholder
    } else {
        props.summary.clone()
    };

    // Shorten wind chill text if present (e.g., "Wind chill minus 26." -> "WC -26°")
    let wind_chill_short = props.wind_chill.as_ref().map(|wc| {
        let wc_lower = wc.to_lowercase();
        if let Some(pos) = wc_lower.find("minus") {
            let after = &wc[pos + 5..];
            for word in after.split_whitespace() {
                let cleaned = word.trim_matches(|c: char| !c.is_numeric());
                if let Ok(num) = cleaned.parse::<i32>() {
                    return format!("WC -{}°", num);
                }
            }
        }
        if let Some(pos) = wc_lower.find("near") {
            let after = &wc[pos + 4..];
            for word in after.split_whitespace() {
                if word.contains("minus") {
                    continue;
                }
                let cleaned = word.trim_matches(|c: char| !c.is_numeric() && c != '-');
                if let Ok(num) = cleaned.parse::<i32>() {
                    return format!("WC {}°", num);
                }
            }
        }
        "".to_string()
    }).filter(|s| !s.is_empty());

    html! {
        <div class="card h-100">
            <div class="card-header text-center p-0 text-body">
                { &props.day_name }
            </div>
            <div class="card-body d-flex flex-column align-items-center gap-1 p-0">
                // Render the emoji icon
                <div class="display-3">
                    { &props.icon }
                </div>

                <div class="text-nowrap text-body fw-bold fs-5">
                    { temp_display }
                </div>

                <div class="text-nowrap text-body fw-bold">
                    { summary_display }
                </div>

                <div class="text-body fw-bold">
                    {format!("POP {}%", props.pop.unwrap_or(0))}
                </div>

                // Show wind chill if available (useful in winter)
                if let Some(ref wc) = wind_chill_short {
                    <div class="text-body text-info">{ wc }</div>
                }
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
                                uv_index={forecast.uv_index.clone()}
                                wind_chill={forecast.wind_chill.clone()}
                                wind_summary={forecast.wind_summary.clone()}
                            />
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

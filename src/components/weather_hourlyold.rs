// src/components/weather_hourly.rs
use yew::{function_component, html, Html, Properties};
use crate::weather::api::HourlyForecast;
use charming::{
    Chart, HtmlRenderer,
    component::{Axis, Grid, Legend, Title},
    element::{AxisType, Tooltip, Trigger},
    series::Line,
    theme::Theme,
};

#[derive(Clone, PartialEq, Properties)]
pub struct WeatherHourlyProps {
    pub forecasts: Vec<HourlyForecast>,
}

#[function_component(WeatherHourly)]
pub fn weather_hourly(props: &WeatherHourlyProps) -> Html {
    // Extract data for the chart
    let times: Vec<String> = props.forecasts.iter()
        .map(|f| f.time.clone())
        .collect();
    
    let temperatures: Vec<i32> = props.forecasts.iter()
        .map(|f| f.temperature)
        .collect();
    
    let precipitation: Vec<u32> = props.forecasts.iter()
        .map(|f| f.pop)
        .collect();

    // Detect dark mode
    let is_dark_mode = web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .and_then(|mq| Some(mq.matches()))
        .unwrap_or(false);

    let text_color = if is_dark_mode { "#ffffff" } else { "#000000" };

    // Create the chart with single y-axis (temperature)
    let chart = Chart::new()
        .title(
            Title::new()
                .text("24-Hour Forecast")
                .text_style(charming::element::TextStyle::new().color(text_color))
        )
        .tooltip(
            Tooltip::new()
                .trigger(Trigger::Axis)
        )
        .legend(
            Legend::new()
                .data(vec!["Temperature (°C)", "Precipitation (%)"])
                .text_style(charming::element::TextStyle::new().color(text_color))
        )
        .grid(
            Grid::new()
                .left("3%")
                .right("4%")
                .bottom("3%")
                .contain_label(true)
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(times)
                .axis_label(charming::element::AxisLabel::new().color(text_color))
        )
        .y_axis(
            Axis::new()
                .type_(AxisType::Value)
                .name("Temperature (°C) / Precipitation (%)")
                .name_text_style(charming::element::TextStyle::new().color(text_color))
                .axis_label(charming::element::AxisLabel::new().color(text_color))
        )
        .series(
            Line::new()
                .name("Temperature (°C)")
                .data(temperatures.into_iter().map(|t| t.into()).collect())
                .smooth(0.3)
        )
        .series(
            Line::new()
                .name("Precipitation (%)")
                .data(precipitation.into_iter().map(|p| p.into()).collect())
                .smooth(0.3)
        );

    // Render the chart
    let theme = if is_dark_mode { Theme::Dark } else { Theme::Default };
    let renderer = HtmlRenderer::new("weather-chart", 800, 400)
        .theme(theme);
    
    let chart_html = renderer.render(&chart).unwrap_or_else(|_| {
        "<div class='alert alert-warning'>Failed to render chart</div>".to_string()
    });

    html! {
        <div class="card mb-3">
            <div class="card-body">
                <div id="weather-chart" dangerously_set_inner_html={chart_html}></div>
            </div>
        </div>
    }
}

use charming::{
    component::{Axis, Grid, Legend},
    element::{
        AxisLabel, AxisTick, AxisType, ItemStyle, LineStyle, MarkArea, MarkAreaData, SplitLine,
        TextStyle,
    },
    series::Line,
    Chart, WasmRenderer,
};
use chrono::{DateTime, Local};
use yew::{function_component, html, use_effect_with, Html, Properties};
use gloo_timers::callback::Timeout;
use gloo_console::log;
use web_sys::window;

use crate::weather::api::WeatherHourly;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Properties)]
pub struct HourlyComponentProps {
    pub data: WeatherHourly,
    pub offset_hours: String,
}

#[function_component]
pub fn HourlyComponent(props: &HourlyComponentProps) -> Html {
    let current_time = Local::now();

    let data = props.data.clone();
    let offset_hours = props.offset_hours.clone();

    // --- NEW: Determine Chart Text Color based on OS preference ---
    let (chart_text_color, split_line_color) = {
        let is_dark_mode = window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
            .map(|mql| mql.matches())
            .unwrap_or(true); // Default to dark mode if check fails

        if is_dark_mode {
            ("#FFFFFF", "#444444") // White text, darker grey split lines for dark theme
        } else {
            ("#000000", "#CCCCCC") // Black text, lighter grey split lines for light theme
        }
    };
    // -----------------------------------------------------------------

    use_effect_with((data.time.clone(), offset_hours.clone()), move |_| {
        log!("HourlyComponent effect triggered");
        
        let mut time = Vec::new();
        let mut temp = Vec::new();
        let mut rain = Vec::new();
        let mut uv: Vec<f32> = Vec::new();

        // ... (data processing logic remains the same) ...
        for (i, time_stamp) in data.time.iter().enumerate() {
            if time.len() >= 48 {
                break;
            }

            let date_str = format!("{}:00{}", time_stamp, offset_hours);
            let date = DateTime::parse_from_rfc3339(&date_str);

            if let Ok(parsed_date) = date {
                if parsed_date >= current_time {
                    time.push(format!("{}", parsed_date.format("%H:%M")));
                    temp.push(data.temperature_2m[i]);
                    rain.push(data.precipitation[i]);
                    uv.push(data.uv_index[i]);
                }
            }
        }

        if !time.is_empty() {
            let chart = Chart::new()
                .legend(
                    Legend::new()
                        .data(vec!["Temperature", "Precipitation", "UV"])
                        // FIX: Use dynamic color
                        .text_style(TextStyle::new().color(chart_text_color)), 
                )
                .x_axis(
                    Axis::new()
                        .type_(AxisType::Category)
                        .data(time.clone())
                        .axis_tick(AxisTick::new().show(false))
                        // FIX: Use dynamic color
                        .axis_label(AxisLabel::new().color(chart_text_color)), 
                )
                .y_axis(
                    Axis::new()
                        .type_(AxisType::Value)
                        // FIX: Use dynamic color
                        .axis_label(AxisLabel::new().color(chart_text_color)) 
                        // FIX: Use dynamic color
                        .split_line(SplitLine::new().line_style(LineStyle::new().color(split_line_color))),
                )
                .y_axis(
                    Axis::new()
                        .type_(AxisType::Value)
                        .axis_label(AxisLabel::new().color("orange")) // Keep orange for UV index axis
                        .split_line(SplitLine::new().line_style(LineStyle::new().opacity(0)))
                        .max(11),
                )
                .series(
                    Line::new()
                        .name("Temperature")
                        .data(temp.clone())
                        .show_symbol(false)
                        // FIX: Use dynamic color
                        .item_style(ItemStyle::new().color(chart_text_color))
                        // FIX: Use dynamic color
                        .line_style(LineStyle::new().width(5).color(chart_text_color))
                        .mark_area(
                            MarkArea::new()
                                .item_style(ItemStyle::new().color("grey"))
                                .data(vec![(
                                    MarkAreaData::new().x_axis("23:00"),
                                    MarkAreaData::new().x_axis("01:00"),
                                )]),
                        ),
                )
                .series(
                    Line::new()
                        .name("Precipitation")
                        .data(rain.clone())
                        .y_axis_index(1)
                        .show_symbol(false)
                        .item_style(ItemStyle::new().color("blue"))
                        .line_style(LineStyle::new().width(3).color("blue")),
                )
                .series(
                    Line::new()
                        .name("UV")
                        .data(uv.clone())
                        .y_axis_index(1)
                        .show_symbol(false)
                        .item_style(ItemStyle::new().color("orange"))
                        .line_style(LineStyle::new().width(3).color("orange")),
                )
                .grid(Grid::new().top(24).left(24).right(24).bottom(20));

            let renderer = WasmRenderer::new(780, 170);
            Timeout::new(100, move || {
                match renderer.render("chart", &chart) {
                    Ok(_) => log!("Chart rendered successfully!"),
                    Err(e) => log!(format!("Chart render error: {:?}", e)),
                }
            }).forget();
        }

        || ()
    });

    html! {
        <>
        <div id="chart" style="width: 780px; height: 170px;"></div>
        </>
    }
}

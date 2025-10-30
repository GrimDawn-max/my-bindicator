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

use crate::context::weather::WeatherHourly;

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

    use_effect_with((data.time.clone(), offset_hours.clone()), move |_| {
        log!("HourlyComponent effect triggered");
        log!(format!("Data time length: {}", data.time.len()));
        log!(format!("Current time: {}", current_time));
        log!(format!("Offset hours: {}", offset_hours));
        
        let mut time = Vec::new();
        let mut temp = Vec::new();
        let mut rain = Vec::new();
        let mut uv: Vec<f32> = Vec::new();

        for (i, time_stamp) in data.time.iter().enumerate() {
            if time.len() >= 48 {
                break;
            }

            // The API returns timestamps like "2025-10-29T00:00"
            // We need to append seconds and timezone offset
            let date_str = format!("{}:00{}", time_stamp, offset_hours);
            log!(format!("Parsing date string: {}", date_str));
            
            let date = DateTime::parse_from_rfc3339(&date_str);

            if let Ok(parsed_date) = date {
                log!(format!("Parsed date: {}, comparing to current: {}", parsed_date, current_time));
                if parsed_date >= current_time {
                    time.push(format!("{}", parsed_date.format("%H:%M")));
                    temp.push(data.temperature_2m[i]);
                    rain.push(data.precipitation[i]);
                    uv.push(data.uv_index[i]);
                }
            } else {
                log!(format!("Failed to parse date: {}", date_str));
            }
        }

        log!(format!("Processed {} time points", time.len()));
        
        if !time.is_empty() {
            log!(format!("Time data: {:?}", &time[..time.len().min(5)]));
            log!(format!("Temp data: {:?}", &temp[..temp.len().min(5)]));
        } else {
            log!("WARNING: No data points to display!");
        }

        if !time.is_empty() {
            let chart = Chart::new()
                .legend(
                    Legend::new()
                        .data(vec!["Temperature", "Precipitation", "UV"])
                        .text_style(TextStyle::new().color("white")),
                )
                .x_axis(
                    Axis::new()
                        .type_(AxisType::Category)
                        .data(time.clone())
                        .axis_tick(AxisTick::new().show(false))
                        .axis_label(AxisLabel::new().color("white")),
                )
                .y_axis(
                    Axis::new()
                        .type_(AxisType::Value)
                        .axis_label(AxisLabel::new().color("white"))
                        .split_line(SplitLine::new().line_style(LineStyle::new().color("grey"))),
                )
                .y_axis(
                    Axis::new()
                        .type_(AxisType::Value)
                        .axis_label(AxisLabel::new().color("orange"))
                        .split_line(SplitLine::new().line_style(LineStyle::new().opacity(0)))
                        .max(11),
                )
                .series(
                    Line::new()
                        .name("Temperature")
                        .data(temp.clone())
                        .show_symbol(false)
                        .item_style(ItemStyle::new().color("white"))
                        .line_style(LineStyle::new().width(5).color("white"))
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

            log!("Chart created, attempting to render");
            let renderer = WasmRenderer::new(780, 170);
            Timeout::new(100, move || {
                log!("Timeout callback executing");
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
        // <p>{"TEST: Weather chart should be above this line"}</p>
        </>
    }
}
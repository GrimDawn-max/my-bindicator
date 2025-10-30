use crate::{
    components::{
        weather_daily::{DailyComponent, DailyComponentProps},
        weather_hourly::HourlyComponent,
    },
    context::weather::WeatherContext,
};
use chrono::{DateTime, Local};
use yew::prelude::*;

#[function_component]
pub fn WeatherComponent() -> Html {
    let weather_ctx = use_context::<WeatherContext>().unwrap();

    // Don't render until data is loaded
    if !weather_ctx.is_loaded {
        return html! {
            <div class="text-dark">
                <p>{"Loading weather data..."}</p>
            </div>
        };
    }

    let weather = weather_ctx.weather.clone();

    // Safety check – ensure data exists
    if weather.hourly.time.is_empty() || weather.daily.time.is_empty() {
        return html! {
            <div class="text-dark">
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

    // Find index of today in daily data
    let today_index = weather.daily.time.iter().position(|time| {
        if let Ok(date) = DateTime::parse_from_rfc3339(&format!("{time}T00:00:00{offset_hours}")) {
            date.date_naive() >= Local::now().date_naive()
        } else {
            false
        }
    }).unwrap_or(0);

    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-dark mt-3">
            {
                weather.daily.time.clone().iter().enumerate()
                    .skip(today_index)
                    .take(7)
                    .map(|(i, time)| {
                        let temp_max = weather.daily.temperature_2m_max[i];
                        let temp_min = weather.daily.temperature_2m_min[i];
                        let precipitation = weather.daily.precipitation_sum[i];
                        let precipitation_probability_max = weather.daily.precipitation_probability_max[i];
                        let code = weather.daily.weather_code[i];

                        let date_utc = DateTime::parse_from_rfc3339(&format!("{time}T00:00:00{offset_hours}")).ok();
                        let local_date = date_utc.map(|d| d.with_timezone(&Local));

                        let weekday_label = local_date
                            .map(|d| d.format("%a").to_string())
                            .unwrap_or_else(|| "?".to_string());

                        let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunrise[i]));
                        let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunset[i]));

                        if let (Some(local_dt), Ok(sr), Ok(ss)) = (local_date, sunrise, sunset) {
                            let props = DailyComponentProps {
                                weather_code: code.to_owned(),
                                temp_max,
                                temp_min,
                                precipitation_sum: precipitation,
                                precipitation_probability_max,
                                date: local_dt.into(),
                                sunrise: sr.into(),
                                sunset: ss.into(),
                            };
                            html! {
                                <div class="text-center">
                                    <p class="fw-bold mb-0">{ weekday_label }</p>
                                    <DailyComponent ..props.clone() />
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    })
                    .collect::<Html>()
            }
            </div>
        </>
    }
}


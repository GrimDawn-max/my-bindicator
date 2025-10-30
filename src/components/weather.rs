use crate::{
    components::{
        weather_daily::{DailyComponent, DailyComponentProps},
        weather_hourly::HourlyComponent,
    },
    context::weather::WeatherContext,
};
use chrono::{DateTime, Local, NaiveDate};
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

    // Safety check
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

    // Convert daily times to local NaiveDate
    let daily_local_dates: Vec<NaiveDate> = weather.daily.time.iter().map(|t| {
        DateTime::parse_from_rfc3339(&format!("{}T00:00:00{offset_hours}", t))
            .ok()
            .map(|dt| dt.with_timezone(&Local).date_naive())
    })
    .flatten()
    .collect();

    // Find today in local time
    let today = Local::now().date_naive();

    // Get the index of the first day >= today
    let start_index = daily_local_dates.iter().position(|&d| d >= today).unwrap_or(0);

    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-dark mt-3">
            {
                (start_index..(start_index + 7).min(weather.daily.time.len()))
                    .map(|i| {
                        let temp_max = weather.daily.temperature_2m_max[i];
                        let temp_min = weather.daily.temperature_2m_min[i];
                        let precipitation = weather.daily.precipitation_sum[i];
                        let precipitation_probability_max = weather.daily.precipitation_probability_max[i];
                        let code = weather.daily.weather_code[i];

                        let date_utc = DateTime::parse_from_rfc3339(&format!("{}T00:00:00{offset_hours}", weather.daily.time[i])).unwrap();
                        let local_date = date_utc.with_timezone(&Local);
                        let weekday_label = local_date.format("%a").to_string();

                        let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunrise[i])).unwrap();
                        let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", weather.daily.sunset[i])).unwrap();

                        let props = DailyComponentProps {
                            weather_code: code.to_owned(),
                            temp_max: temp_max.to_owned(),
                            temp_min: temp_min.to_owned(),
                            precipitation_sum: precipitation.to_owned(),
                            precipitation_probability_max: precipitation_probability_max.to_owned(),
                            date: local_date.into(),
                            sunrise: sunrise.into(),
                            sunset: sunset.into(),
                        };

                        html! {
                            <div class="text-center">
                                <p class="fw-bold mb-0">{ weekday_label }</p>
                                <DailyComponent ..props.clone() />
                            </div>
                        }
                    })
                    .collect::<Html>()
            }
            </div>
        </>
    }
}


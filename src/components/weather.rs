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

    // Find index of today or first future day
    let today_index = weather.daily.time.iter().position(|time| {
        let date = DateTime::parse_from_rfc3339(&format!("{}T00:00:00{offset_hours}", time));
        date.map_or(false, |dt| dt.date_naive() >= Local::now().date_naive())
    }).unwrap_or(0);

    // Slice arrays from today onward
    let daily_times = &weather.daily.time[today_index..];
    let daily_temp_max = &weather.daily.temperature_2m_max[today_index..];
    let daily_temp_min = &weather.daily.temperature_2m_min[today_index..];
    let daily_precip_sum = &weather.daily.precipitation_sum[today_index..];
    let daily_precip_prob = &weather.daily.precipitation_probability_max[today_index..];
    let daily_weather_code = &weather.daily.weather_code[today_index..];
    let daily_sunrise = &weather.daily.sunrise[today_index..];
    let daily_sunset = &weather.daily.sunset[today_index..];

    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-dark mt-3">
            {
                daily_times.iter().enumerate()
                    .take(7) // Only 7 days
                    .map(|(i, time)| {
                        let temp_max = daily_temp_max[i];
                        let temp_min = daily_temp_min[i];
                        let precipitation = daily_precip_sum[i];
                        let precipitation_probability_max = daily_precip_prob[i];
                        let code = daily_weather_code[i];

                        let date_utc = DateTime::parse_from_rfc3339(&format!("{time}T00:00:00{offset_hours}")).ok();
                        let local_date = date_utc.map(|d| d.with_timezone(&Local));

                        let weekday_label = local_date
                            .map(|d| d.format("%a").to_string())
                            .unwrap_or_else(|| "?".to_string());

                        let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", daily_sunrise[i]));
                        let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", daily_sunset[i]));

                        if let (Some(local_dt), Ok(sr), Ok(ss)) = (local_date, sunrise, sunset) {
                            let props = DailyComponentProps {
                                weather_code: code.to_owned(),
                                temp_max: temp_max.to_owned(),
                                temp_min: temp_min.to_owned(),
                                precipitation_sum: precipitation.to_owned(),
                                precipitation_probability_max: precipitation_probability_max.to_owned(),
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


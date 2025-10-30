use crate::{
    components::{
        weather_daily::{DailyComponent, DailyComponentProps},
        weather_hourly::HourlyComponent,
    },
    context::weather::WeatherContext,
};
use chrono::{DateTime, Local, TimeZone};
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

    // Find index for today
    let today_index = weather.daily.time.iter().position(|time| {
        let dt = DateTime::parse_from_rfc3339(&format!("{time}T00:00:00{offset_hours}")).ok();
        dt.map_or(false, |d| d.with_timezone(&Local).date_naive() >= Local::now().date_naive())
    }).unwrap_or(0);

    // Rotate daily arrays so they start from today
    let rotate = |v: Vec<_>| {
        let mut v = v;
        v.rotate_left(today_index);
        v.into_iter().take(7).collect::<Vec<_>>()
    };

    let daily_times = rotate(weather.daily.time.clone());
    let daily_max = rotate(weather.daily.temperature_2m_max.clone());
    let daily_min = rotate(weather.daily.temperature_2m_min.clone());
    let daily_precip = rotate(weather.daily.precipitation_sum.clone());
    let daily_precip_prob = rotate(weather.daily.precipitation_probability_max.clone());
    let daily_code = rotate(weather.daily.weather_code.clone());
    let daily_sunrise = rotate(weather.daily.sunrise.clone());
    let daily_sunset = rotate(weather.daily.sunset.clone());

    html! {
        <>
            <HourlyComponent data={weather.hourly.clone()} offset_hours={offset_hours.clone()} />
            <div class="card-group text-dark mt-3">
                { for (0..7).map(|i| {
                    let date_utc = DateTime::parse_from_rfc3339(&format!("{}T00:00:00{offset_hours}", daily_times[i])).ok();
                    let local_date = date_utc.map(|d| d.with_timezone(&Local));
                    let weekday_label = local_date.map(|d| d.format("%a").to_string()).unwrap_or_else(|| "?".to_string());

                    let sunrise = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", daily_sunrise[i]));
                    let sunset = DateTime::parse_from_rfc3339(&format!("{}:00{offset_hours}", daily_sunset[i]));

                    if let (Some(local_dt), Ok(sr), Ok(ss)) = (local_date, sunrise, sunset) {
                        let props = DailyComponentProps {
                            weather_code: daily_code[i].to_owned(),
                            temp_max: daily_max[i].to_owned(),
                            temp_min: daily_min[i].to_owned(),
                            precipitation_sum: daily_precip[i].to_owned(),
                            precipitation_probability_max: daily_precip_prob[i].to_owned(),
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
                }) }
            </div>
        </>
    }
}


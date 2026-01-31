use yew::prelude::*;
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use crate::weather::api::{fetch_weather_data, WeatherData};
use crate::components::weather_hourly::WeatherHourly;
use crate::components::weather_daily::WeatherDaily;

#[function_component(Weather)]
pub fn weather() -> Html {
    let weather_data = use_state(|| None::<WeatherData>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let retry_count = use_state(|| 0);

    {
        let weather_data = weather_data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let retry_count = retry_count.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);

                match fetch_with_retry(&retry_count).await {
                    Ok(data) => {
                        log!("✓ Weather data loaded successfully");
                        weather_data.set(Some(data));
                        loading.set(false);
                        retry_count.set(0);
                    }
                    Err(e) => {
                        log!(&format!("✗ Failed to load weather data: {}", e));
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    let on_retry = {
        let weather_data = weather_data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let retry_count = retry_count.clone();

        Callback::from(move |_| {
            let weather_data = weather_data.clone();
            let loading = loading.clone();
            let error = error.clone();
            let retry_count = retry_count.clone();

            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);

                match fetch_with_retry(&retry_count).await {
                    Ok(data) => {
                        log!("✓ Weather data loaded successfully on retry");
                        weather_data.set(Some(data));
                        loading.set(false);
                        retry_count.set(0);
                    }
                    Err(e) => {
                        log!(&format!("✗ Retry failed: {}", e));
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="weather-container">
            if *loading {
                <div class="alert alert-info d-flex align-items-center">
                    <div class="spinner-border spinner-border-sm me-2" role="status">
                        <span class="visually-hidden">{"Loading..."}</span>
                    </div>
                    <div>
                        {"Loading weather data..."}
                        if *retry_count > 0 {
                            <small class="d-block mt-1">
                                {format!("Attempt {}/3", *retry_count)}
                            </small>
                        }
                    </div>
                </div>
            } else if let Some(err_msg) = (*error).as_ref() {
                <div class="alert alert-warning">
                    <strong>{"⚠️ Weather temporarily unavailable"}</strong>
                    <p class="mb-2 mt-2 small">{err_msg}</p>
                    <button class="btn btn-sm btn-outline-secondary" onclick={on_retry}>
                        {"🔄 Retry"}
                    </button>
                </div>
            } else if let Some(data) = (*weather_data).as_ref() {
                <>
                    // Weather warnings (if any)
                    if !data.warnings.is_empty() {
                        <div class="mb-3">
                            {data.warnings.iter().map(|warning| {
                                let alert_class = match warning.alert_level.as_str() {
                                    "red" => "alert-danger",
                                    "orange" => "alert-warning",
                                    "yellow" => "alert-warning",
                                    _ => "alert-info",
                                };
                                html! {
                                    <div class={format!("alert {} py-2", alert_class)}>
                                        <strong>{"⚠️ "}{&warning.description}</strong>
                                        if !warning.url.is_empty() {
                                            <a href={warning.url.clone()} target="_blank" class="ms-2 small">
                                                {"Details →"}
                                            </a>
                                        }
                                    </div>
                                }
                            }).collect::<Html>()}
                        </div>
                    }

                    // Current conditions
                    <div class="card mb-3 current-weather">
                        <div class="card-body">
                            <h5 class="card-title">
                                {"Current Conditions"}
                                if !data.current.station.is_empty() {
                                    <small class="text-muted ms-2">{format!("({})", data.current.station)}</small>
                                }
                            </h5>
                            <div class="row">
                                <div class="col-md-6">
                                    <div class="d-flex align-items-center mb-2">
                                        <span class="weather-icon me-2" style="font-size: 3rem;">{&data.current.icon}</span>
                                        <div>
                                            <h2 class="mb-0">{format!("{}°C", data.current.temperature)}</h2>
                                            <p class="mb-0">{&data.current.condition}</p>
                                            if let Some(wc) = data.current.wind_chill {
                                                <p class="mb-0 text-info small">{format!("Feels like {}°C", wc)}</p>
                                            }
                                        </div>
                                    </div>
                                </div>
                                <div class="col-md-6">
                                    <div class="small ps-3">
                                        // Wind at top
                                        <div class="mb-2">
                                            {"Wind: "}<strong>{format!("{} km/h {}", data.current.wind_speed, data.current.wind_direction)}</strong>
                                            if let Some(gust) = data.current.wind_gust {
                                                <span class="text-warning">{format!(" (gusts {})", gust)}</span>
                                            }
                                        </div>

                                        // Row 1: Air Quality
                                        if let Some(ref aq) = data.current.air_quality {
                                            <div class="mb-2">
                                                <div class="mb-1 text-nowrap">
                                                    {"Air Quality: "}
                                                    <strong class={get_aqhi_color_class(aq.index)}>{&aq.category}</strong>{" "}
                                                    <span class="badge bg-secondary">{format!("{:.0}", aq.index)}</span>
                                                </div>
                                                <div style="max-width: 180px;">
                                                    <div class="position-relative" style="height: 8px; border-radius: 4px; background: linear-gradient(to right, #00e400 0%, #00e400 20%, #ffff00 20%, #ffff00 40%, #ff7e00 40%, #ff7e00 60%, #ff0000 60%, #ff0000 80%, #8f3f97 80%, #8f3f97 100%);">
                                                        <div style={format!("position: absolute; top: -3px; left: calc({}% - 6px); width: 12px; height: 14px; background: white; border: 2px solid #333; border-radius: 3px;", (aq.index / 10.0 * 100.0).min(100.0))}></div>
                                                    </div>
                                                    <div class="d-flex justify-content-between" style="font-size: 0.6rem;">
                                                        <span>{"1"}</span>
                                                        <span>{"3"}</span>
                                                        <span>{"6"}</span>
                                                        <span>{"10+"}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        }

                                        // Row 2: Sunrise | Sunset | Humidity
                                        <div class="d-flex gap-3 mb-1">
                                            if let Some(ref sun) = data.sun {
                                                <span class="text-nowrap">{"☀️ "}<strong>{&sun.sunrise}</strong></span>
                                                <span class="text-nowrap">{"🌙 "}<strong>{&sun.sunset}</strong></span>
                                            }
                                            <span class="text-nowrap">{"💧 "}<strong>{format!("{}%", data.current.humidity)}</strong></span>
                                        </div>

                                        // Row 3: Dew Point | Visibility | Pressure (with trend arrow)
                                        <div class="d-flex gap-3 mb-1">
                                            <span class="text-nowrap">{"Dew: "}<strong>{format!("{:.1}°C", data.current.dewpoint)}</strong></span>
                                            if let Some(vis) = data.current.visibility {
                                                <span class="text-nowrap">{"Vis: "}<strong>{format!("{:.0} km", vis)}</strong></span>
                                            }
                                            <span class="text-nowrap">
                                                {"Press: "}<strong>{format!("{:.1} kPa", data.current.pressure)}</strong>
                                                {get_pressure_arrow(&data.current.pressure_tendency)}
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Hourly forecast chart
                    <WeatherHourly forecasts={data.hourly.clone()} />

                    // Daily forecast cards
                    <WeatherDaily forecasts={data.daily.clone()} />
                </>
            }
        </div>
    }
}

fn get_aqhi_color_class(index: f32) -> &'static str {
    match index.round() as u32 {
        1..=3 => "text-success",
        4..=6 => "text-warning",
        7..=10 => "text-danger",
        _ => "text-danger",
    }
}

fn get_pressure_arrow(tendency: &Option<String>) -> Html {
    if let Some(t) = tendency {
        let t_lower = t.to_lowercase();
        if t_lower.contains("rising") || t_lower.contains("up") {
            html! { <span class="pressure-rising">{" ▲"}</span> }
        } else if t_lower.contains("falling") || t_lower.contains("down") {
            html! { <span class="pressure-falling">{" ▼"}</span> }
        } else if t_lower.contains("steady") || t_lower.contains("stable") {
            html! { <span class="pressure-steady">{" —"}</span> }
        } else {
            html! {}
        }
    } else {
        html! {}
    }
}

async fn fetch_with_retry(retry_count: &UseStateHandle<u32>) -> Result<WeatherData, String> {
    const MAX_ATTEMPTS: u32 = 3;
    let mut attempts = 0;

    loop {
        attempts += 1;
        retry_count.set(attempts);

        match fetch_weather_data().await {
            Ok(data) => return Ok(data),
            Err(e) if attempts < MAX_ATTEMPTS => {
                // Exponential backoff: 2s, 4s, 8s
                let delay_ms = 2u32.pow(attempts) * 1000;
                log!(
                    &format!("Attempt {}/{} failed: {}. Retrying in {}ms...",
                    attempts,
                    MAX_ATTEMPTS,
                    e,
                    delay_ms)
                );
                TimeoutFuture::new(delay_ms).await;
            }
            Err(e) => {
                return Err(format!(
                    "Failed after {} attempts. {}",
                    MAX_ATTEMPTS,
                    e
                ));
            }
        }
    }
}

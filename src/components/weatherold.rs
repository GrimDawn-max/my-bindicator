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
                        log!("✗ Failed to load weather data: {}", e);
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
                        log!("✗ Retry failed: {}", e);
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
                    // Current conditions
                    <div class="card mb-3 current-weather">
                        <div class="card-body">
                            <h5 class="card-title">{"Current Conditions"}</h5>
                            <div class="row">
                                <div class="col-md-6">
                                    <div class="d-flex align-items-center mb-2">
                                        <span class="weather-icon me-2" style="font-size: 3rem;">{&data.current.icon}</span>
                                        <div>
                                            <h2 class="mb-0">{format!("{}°C", data.current.temperature)}</h2>
                                            <p class="mb-0">{&data.current.condition}</p>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-md-6">
                                    <div class="weather-details small">
                                        <div class="d-flex justify-content-between mb-1">
                                            <span>{"Humidity:"}</span>
                                            <strong>{format!("{}%", data.current.humidity)}</strong>
                                        </div>
                                        <div class="d-flex justify-content-between mb-1">
                                            <span>{"Wind:"}</span>
                                            <strong>{format!("{} km/h {}", data.current.wind_speed, data.current.wind_direction)}</strong>
                                        </div>
                                        <div class="d-flex justify-content-between mb-1">
                                            <span>{"Pressure:"}</span>
                                            <strong>{format!("{:.1} kPa", data.current.pressure)}</strong>
                                        </div>
                                        <div class="d-flex justify-content-between mb-1">
                                            <span>{"Visibility:"}</span>
                                            <strong>{format!("{:.1} km", data.current.visibility)}</strong>
                                        </div>
                                        <div class="d-flex justify-content-between">
                                            <span>{"Dewpoint:"}</span>
                                            <strong>{format!("{:.1}°C", data.current.dewpoint)}</strong>
                                        </div>
                                        if let Some(ref aq) = data.current.air_quality {
                                            <div class="d-flex justify-content-between mt-1">
                                                <span>{"Air Quality:"}</span>
                                                <strong>{format!("{} ({})", aq.index, aq.category)}</strong>
                                            </div>
                                        }
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
                    "Attempt {}/{} failed: {}. Retrying in {}ms...",
                    attempts,
                    MAX_ATTEMPTS,
                    e,
                    delay_ms
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

// src/context/weather.rs - COMPLETE REPLACEMENT

use std::rc::Rc;
use yew::prelude::*;
use gloo_console::log;
use serde::{Deserialize, Serialize};
use yew_hooks::use_interval;
use crate::weather::api::{WeatherData, fetch_weather_data};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherContextData {
    pub weather: Option<WeatherData>,
    pub loading: bool,
    pub error: Option<String>,
}

impl Default for WeatherContextData {
    fn default() -> Self {
        Self {
            weather: None,
            loading: true,
            error: None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct WeatherContext {
    pub data: Rc<WeatherContextData>,
    pub refresh: Callback<()>,
}

#[derive(Properties, PartialEq)]
pub struct WeatherProviderProps {
    pub children: Children,
}

#[function_component(WeatherProvider)]
pub fn weather_provider(props: &WeatherProviderProps) -> Html {
    let state = use_state(WeatherContextData::default);
    
    // Refresh callback
    let refresh = {
        let state = state.clone();
        Callback::from(move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                state.set(WeatherContextData {
                    weather: None,
                    loading: true,
                    error: None,
                });
                
                match fetch_weather_with_retry().await {
                    Ok(weather) => {
                        state.set(WeatherContextData {
                            weather: Some(weather),
                            loading: false,
                            error: None,
                        });
                    }
                    Err(e) => {
                        log!(&format!("Error fetching weather: {}", e));
                        state.set(WeatherContextData {
                            weather: None,
                            loading: false,
                            error: Some(e),
                        });
                    }
                }
            });
        })
    };

    // Initial load
    {
        let refresh = refresh.clone();
        use_effect_with((), move |_| {
            refresh.emit(());
            || ()
        });
    }

    // Auto-refresh every hour
    {
        let refresh = refresh.clone();
        use_interval(
            move || {
                refresh.emit(());
            },
            3600000, // 1 hour in milliseconds
        );
    }

    let context = WeatherContext {
        data: Rc::new((*state).clone()),
        refresh,
    };

    html! {
        <ContextProvider<WeatherContext> context={context}>
            {props.children.clone()}
        </ContextProvider<WeatherContext>>
    }
}

async fn fetch_weather_with_retry() -> Result<WeatherData, String> {
    // Single attempt - api.rs already has built-in fallback proxies
    fetch_weather_data().await
}

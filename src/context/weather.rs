use std::rc::Rc;
use core::time::Duration;
use gloo_timers::future::sleep;

use gloo_console::{log, warn}; 
use yew::{platform::spawn_local, prelude::*};
use yew_hooks::use_interval;

use crate::{
    context::location::LocationContext,
    weather::api::EnvironmentCanadaClient,
    weather::models::WeatherData,
};

// Retry constants
#[allow(dead_code)]
const MAX_RETRIES: u8 = 3;
#[allow(dead_code)]
const RETRY_DELAY_MS: u64 = 2000; // 2 seconds delay between retries

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct WeatherCtx {
    pub is_loaded: bool,
    pub weather: WeatherData,
}

impl Reducible for WeatherCtx {
    type Action = WeatherData;

    fn reduce(self: Rc<Self>, data: Self::Action) -> Rc<Self> {
        WeatherCtx {
            is_loaded: true,
            weather: data,
        }
        .into()
    }
}

#[allow(dead_code)]
pub type WeatherContext = UseReducerHandle<WeatherCtx>;

#[allow(dead_code)]
#[derive(Properties, Debug, PartialEq)]
pub struct WeatherProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn WeatherProvider(props: &WeatherProviderProps) -> Html {
    let weather = use_reducer(|| WeatherCtx {
        is_loaded: false,
        weather: WeatherData {
            ..Default::default()
        },
    });

    let _location_ctx = use_context::<LocationContext>().unwrap(); 

    let client = EnvironmentCanadaClient::toronto();

    let weather_clone = weather.clone();
    let client_clone_on_mount = client.clone();
    
    // Initial data fetch on mount
    use_effect_with((), move |_| {
        spawn_local(async move {
            let data = fetch_weather_with_retry(&client_clone_on_mount).await;
            weather_clone.dispatch(data);
        });
        || ()
    });

    // Interval logic for hourly updates
    let update_every_millis = 1000 * 60 * 60; // 1 hour
    let client_clone_on_interval = client.clone();
    let weather_clone_on_interval = weather.clone();
    
    use_interval(
        move || {
            log!("In use interval: Attempting weather refresh.");
            
            let client_clone = client_clone_on_interval.clone();
            let weather_clone = weather_clone_on_interval.clone();
            
            spawn_local(async move {
                let data = fetch_weather_with_retry(&client_clone).await;
                weather_clone.dispatch(data);
            });
        },
        update_every_millis,
    );
    
    html! {
        <ContextProvider<WeatherContext> context={weather}>
            {props.children.clone()}
        </ContextProvider<WeatherContext>>
    }
}

/// Attempts to fetch weather data from the Environment Canada client with retries.
#[allow(dead_code)]
async fn fetch_weather_with_retry(client: &EnvironmentCanadaClient) -> WeatherData {
    for attempt in 0..MAX_RETRIES {
        let result = client.fetch_weather().await;
        
        match result {
            Ok(data) => {
                log!(format!("Weather fetch attempt {} succeeded.", attempt + 1));
                if !data.location.is_empty() {
                    return data; 
                } else {
                    warn!(format!("Attempt {} failed (Data empty or invalid structure).", attempt + 1));
                }
            }
            Err(e) => {
                warn!(format!("Attempt {} failed (Network/Parse error: {}).", attempt + 1, e));
            }
        }
        
        if attempt < MAX_RETRIES - 1 {
            warn!(format!("Retrying in {}ms...", RETRY_DELAY_MS));
            sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
    }

    warn!("Failed to load weather data after all retries. Returning empty data.");
    WeatherData::default()
}
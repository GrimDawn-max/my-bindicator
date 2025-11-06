use std::rc::Rc;
use core::time::Duration; // RESTORED: For Duration
use gloo_timers::future::sleep; // RESTORED: For sleep function

use gloo_console::{log, warn}; 
use serde::Deserialize;
use yew::{platform::spawn_local, prelude::*};
use yew_hooks::use_interval;

use crate::{
    context::location::LocationContext,
    weather::api::EnvironmentCanadaClient, // ADDED: Import the EC client
    weather::models::WeatherData, // ADDED: Import the EC WeatherData model
};

use super::location::Coordinates; 

// --- NEW CONSTANTS FOR RETRY LOGIC ---
const MAX_RETRIES: u8 = 3;
const RETRY_DELAY_MS: u64 = 2000; // 2 seconds delay between retries
// ------------------------------------

// Easier to deal with a single 'variable'
#[derive(Debug, PartialEq, Clone)]
pub struct WeatherCtx {
    pub is_loaded: bool,
    pub weather: WeatherData,
}

// REMOVED: Open-Meteo data structures (WeatherDaily, WeatherHourly)
// The correct WeatherData should now come from src/weather/models.rs
// NOTE: Ensure your src/weather/models.rs contains the WeatherData struct definition.

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

pub type WeatherContext = UseReducerHandle<WeatherCtx>;

#[derive(Properties, Debug, PartialEq)]
pub struct WeatherProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn WeatherProvider(props: &WeatherProviderProps) -> Html {
    let weather = use_reducer(|| WeatherCtx {
        is_loaded: false,
        // Since WeatherData now derives Default (in models.rs), this is fine
        weather: WeatherData {
            ..Default::default()
        },
    });

    // location_ctx is not used directly by the EC client, but kept for context
    let _location_ctx = use_context::<LocationContext>().unwrap(); 

    // --- EC CLIENT SETUP ---
    let client = EnvironmentCanadaClient::toronto();
    // -----------------------

    let weather_clone = weather.clone();
    let client_clone_on_mount = client.clone();
    
    // Initial data fetch using use_effect_with(()) to run once on mount
    use_effect_with((), move |_| {
        // Run once on component mount
        spawn_local(async move {
            let data = fetch_weather_with_retry(&client_clone_on_mount).await;
            weather_clone.dispatch(data);
        });
        || ()
    });


    // Interval logic for hourly updates (uncommented and updated)
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

// --- NEW RETRY IMPLEMENTATION FOR EC CLIENT ---

/// Attempts to fetch weather data from the Environment Canada client with retries.
async fn fetch_weather_with_retry(client: &EnvironmentCanadaClient) -> WeatherData {
    for attempt in 0..MAX_RETRIES {
        let result = client.fetch_weather().await;
        
        match result {
            Ok(data) => {
                log!(format!("Weather fetch attempt {} succeeded.", attempt + 1));
                // A successful parse means we got data, but let's check basic validity.
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
            // Delay only if it's not the last attempt
            warn!(format!("Retrying in {}ms...", RETRY_DELAY_MS));
            sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
    }

    // After all retries fail, return a default/empty structure
    warn!("Failed to load weather data after all retries. Returning empty data.");
    return WeatherData::default();
}


// The entire Open-Meteo function is commented out (as per previous step).
/*
async fn fetch_weather(coordinates: Coordinates) -> WeatherApiData {
// ... old Open-Meteo logic ...
}
*/
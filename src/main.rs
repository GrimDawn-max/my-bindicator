mod components;
use components::carousel::Carousel;
use components::clock::ClockComponent;
use components::dim::DimComponent;
use components::location_input::LocationInput;
use components::{bin::BinComponent, carousel::CarouselItem};
mod context;
use context::{bussin::BusProvider, location::LocationProvider, weather::WeatherProvider};
mod utils;
// Environment Canada weather module
mod weather;
use weather::api::WeatherData;
// Import the Weather component instead of WeatherDisplay
use components::weather::Weather;

// === NEW IMPORTS FOR THEME SWITCHING ===
use yew::{function_component, html, use_state, use_context, Callback, Html, use_effect_with, hook};
use web_sys::{window, MediaQueryList}; 

// === NEW CUSTOM HOOK: use_theme_switcher (Step 2) ===
#[hook]
fn use_theme_switcher() {
    // This effect runs once when the component mounts.
    use_effect_with((), |_| {
        // Safely get references to the browser's environment
        let window = window().expect("window not available");
        let document = window.document().expect("document not available");
        // We interact directly with the <body> element
        let body = document.body().expect("body not available");
        
        // Function to apply the correct theme based on the query result
        let apply_theme = |mql: MediaQueryList| {
            if mql.matches() {
                // System is dark (usually night/user preference)
                body.set_attribute("data-bs-theme", "dark").unwrap();
            } else {
                // System is light (usually day)
                body.set_attribute("data-bs-theme", "light").unwrap();
            }
        };
        
        // Check the theme preference immediately
        let media_query_list = window.match_media("(prefers-color-scheme: dark)");
        if let Ok(Some(mql)) = media_query_list {
            // Apply theme based on current OS preference
            apply_theme(mql.clone()); 
        } else {
            // Fallback: If media query fails for some reason, default to light
            body.set_attribute("data-bs-theme", "light").unwrap();
        }
        
        // The cleanup closure is empty since we're not setting up persistent listeners
        || {} 
    });
}

#[function_component]
pub fn App() -> Html {
    // === NEW: Call the custom hook (Step 3) ===
    use_theme_switcher();
    
    html! {
        // Wrap everything in WeatherProvider so weather data is available throughout
        <WeatherProvider>
            <AppContent />
        </WeatherProvider>
    }
}

#[function_component]
fn AppContent() -> Html {
    // Get weather data from context
    let weather_context = use_context::<context::weather::WeatherContext>()
        .expect("WeatherContext not found");
    
    html! {
        // CORRECTION HERE: Added the 'text-body' class to inherit theme colors.
        <div id="app" class="d-flex flex-column justify-content-between p-2 text-body" style="overflow: hidden;">
            <DimComponent/>
            <div class="d-flex justify-content-between">
                // BinComponent now receives weather data from context
                <BinComponent weather={weather_context.data.weather.clone()} />
                <ClockComponent/>
            </div>
            <LocationProvider>
                <Carousel id="main">
                    // Weather component handles its own loading
                    <CarouselItem active={true}>
                        <Weather />
                    </CarouselItem>
                    
                    <CarouselItem active={false}>
                        <LocationInput />
                    </CarouselItem>
                    
                    <CarouselItem active={false}>
                        <BusProvider>
                        </BusProvider>
                    </CarouselItem>
                </Carousel>
            </LocationProvider>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

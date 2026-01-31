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
// Import the Weather component instead of WeatherDisplay
use components::weather::Weather;

use yew::{function_component, html, use_context, Html, use_effect_with, hook};
use web_sys::window;

#[hook]
fn use_theme_switcher() {
    use_effect_with((), |_| {
        let window = window().expect("window not available");
        let document = window.document().expect("document not available");
        let body = document.body().expect("body not available");

        // Check system preference
        if let Ok(Some(mq)) = window.match_media("(prefers-color-scheme: dark)") {
            if mq.matches() {
                let _ = body.set_attribute("data-bs-theme", "dark");
            } else {
                let _ = body.set_attribute("data-bs-theme", "light");
            }
        }

        || {}
    });
}

#[function_component]
pub fn App() -> Html {
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
        <div id="app" class="d-flex flex-column justify-content-between p-2" style="overflow: hidden;">
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

mod components;
use components::carousel::Carousel;
use components::clock::ClockComponent;
use components::dim::DimComponent;
use components::location_input::LocationInput;
use components::weather::WeatherComponent;
use components::{bin::BinComponent, carousel::CarouselItem};

mod context;
use context::{bussin::BusProvider, location::LocationProvider, weather::WeatherProvider};

mod utils;

// ADD THIS: New Environment Canada weather module
mod weather;
use weather::{WeatherDisplay, WeatherData};

use yew::{function_component, html, use_state, Callback, Html};

#[function_component]
pub fn App() -> Html {
    // State to hold Environment Canada weather data
    let ec_weather = use_state(|| None::<WeatherData>);
    
    // Callback to receive weather data from WeatherDisplay
    let on_weather_loaded = {
        let ec_weather = ec_weather.clone();
        Callback::from(move |data: WeatherData| {
            ec_weather.set(Some(data));
        })
    };
    
    html! {
        <div id="app" class="d-flex flex-column justify-content-between p-2" style="overflow: hidden;">
            <DimComponent/>
            <div class="d-flex justify-content-between">
                // BinComponent now receives weather data
                <BinComponent weather={(*ec_weather).clone()} />
                <ClockComponent/>
            </div>
            <LocationProvider>
                <Carousel id="main">
                    // Environment Canada Weather (new)
                    <CarouselItem active={true}>
                        <WeatherDisplay {on_weather_loaded} />
                    </CarouselItem>
                    
                    // Your existing weather component (keep if you want both)
                    <CarouselItem active={false}>
                        <WeatherProvider>
                            <WeatherComponent/>
                        </WeatherProvider>
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
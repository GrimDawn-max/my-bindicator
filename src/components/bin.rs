use chrono::prelude::*;
use chrono::{DateTime, Local, Weekday};
use futures_util::StreamExt;
use std::time::Duration;
use yew::platform::time::interval;
use yew::{function_component, html, AttrValue, Component, Context, Html, Properties};

use crate::weather::api::WeatherData;

const REFRESH_HOURS: u64 = 1;

pub enum BinVariation {
    Yellow,
    None,
}

// Yard waste collection season dates - update these each year
const YARD_WASTE_START_MONTH: u32 = 3;
const YARD_WASTE_START_DAY: u32 = 20;
const YARD_WASTE_END_MONTH: u32 = 12;
const YARD_WASTE_END_DAY: u32 = 11;

// Christmas tree collection dates - update these each year
const CHRISTMAS_TREE_START_MONTH: u32 = 1;
const CHRISTMAS_TREE_START_DAY: u32 = 6;
const CHRISTMAS_TREE_END_MONTH: u32 = 1;
const CHRISTMAS_TREE_END_DAY: u32 = 31;

// Check if we're in yard waste season
pub fn is_yard_waste_season() -> bool {
    let current = get_today();
    let year = current.year();
    
    // Toronto yard waste collection season
    let season_start = Local.with_ymd_and_hms(year, YARD_WASTE_START_MONTH, YARD_WASTE_START_DAY, 0, 0, 0).unwrap();
    let season_end = Local.with_ymd_and_hms(year, YARD_WASTE_END_MONTH, YARD_WASTE_END_DAY, 23, 59, 59).unwrap();
    
    current >= season_start && current <= season_end
}

// Check if we're in Christmas tree collection period
pub fn is_christmas_tree_season() -> bool {
    let current = get_today();
    let year = current.year();
    
    // Toronto Christmas tree collection (January 6-31)
    let season_start = Local.with_ymd_and_hms(year, CHRISTMAS_TREE_START_MONTH, CHRISTMAS_TREE_START_DAY, 0, 0, 0).unwrap();
    let season_end = Local.with_ymd_and_hms(year, CHRISTMAS_TREE_END_MONTH, CHRISTMAS_TREE_END_DAY, 23, 59, 59).unwrap();
    
    current >= season_start && current <= season_end
}

// Blue and Black/Brown bins alternate every week (based on 2-week cycle)
pub fn get_alternate_bin() -> BinVariation {
    // This date needs to be updated annually to align with the current cycle
    let known_yellow_bin_day = Local.with_ymd_and_hms(2025, 10, 16, 0, 0, 0).unwrap();
    let diff = get_today() - known_yellow_bin_day;

    let wat = diff.num_days() % 14;

    if wat != 0 && wat <= 7 {
        return BinVariation::None; // Will display Blue bin
    }
    return BinVariation::Yellow; // Will display Black and Brown bins
}

pub fn get_today() -> DateTime<Local> {
    let current: DateTime<Local> = Local::now();
    return current;
}

#[derive(Properties, PartialEq)]
pub struct BinComponentProps {
    #[prop_or_default]
    pub weather: Option<WeatherData>,
}

pub struct BinComponent {
    current_time: DateTime<Local>,
}

pub enum BinComponentMsg {
    ClockTicked(DateTime<Local>),
}

impl Component for BinComponent {
    type Message = BinComponentMsg;
    type Properties = BinComponentProps; 

    fn create(ctx: &Context<Self>) -> Self {
        let time_steam =
            interval(Duration::from_secs(60 * 60 * REFRESH_HOURS)).map(|_| get_today());
        ctx.link()
            .send_stream(time_steam.map(BinComponentMsg::ClockTicked));

        Self {
            current_time: get_today(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BinComponentMsg::ClockTicked(current_time) => {
                self.current_time = current_time;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let show_brown_bin = is_yard_waste_season();
        let show_christmas_tree = is_christmas_tree_season();
        
        // Calculate days until pickup
        let days_until_pickup = (3 + 7 - self.current_time.weekday().num_days_from_monday()) % 7;
        let days_text = if days_until_pickup == 0 {
            "Today".to_string()
        } else if days_until_pickup == 1 {
            "Tomorrow".to_string()
        } else {
            format!("{} days", days_until_pickup)
        };
        
        // Get day name for forecast lookup
        let pickup_date = self.current_time + chrono::Duration::days(days_until_pickup as i64);
        let day_name = pickup_date.format("%A").to_string(); // "Thursday", "Friday", etc.
        
        // Get forecast for pickup day
        let forecast = ctx.props().weather.as_ref()
            .and_then(|w| w.get_forecast_for_day(&day_name));
        
        html! {
            <div class="d-flex align-items-center">
                // Only Green bin is always displayed
                <BinImage src="GreenBin.png" alt="Green Bin" />

                // Alternating Blue vs Black and Brown bins
                {
                    match get_alternate_bin() {
                        BinVariation::Yellow => html! { 
                            <> 
                                <BinImage src="GarbageBin.png" alt="Garbage Bin" />
                                // Brown bin only shown during yard waste season
                                if show_brown_bin {
                                    // FIX: Explicitly set height and width to maintain correct aspect ratio on mobile
                                    <BinImage 
                                        src="YardWaste.png" 
                                        alt="Yard Waste" 
                                        size_style="height: 4rem; width: 2.9rem;"
                                    />
                                }
                            </> 
                        },
                        BinVariation::None => html! { <BinImage src="BlueBin.png" alt="Blue Bin" /> }
                    }
                }

                // Christmas tree icon during collection period
                if show_christmas_tree {
                    <BinImage src="Christmastree.png" alt="Christmas Tree" />
                }

                <div class="fs-1 fw-bold text-body"> 
                    if self.current_time.weekday() == Weekday::Thu {
                        {"BIN DAY TODAY!!"}
                    } else {
                        {days_text}
                    }
                </div>
                
                // Weather info display for pickup day forecast
                {
                    if let Some(f) = forecast {
                        html! {
                            <div class="ms-3 text-body">
                                <div class="fs-5">
                                    {&f.icon}{" "}{&f.summary}
                                </div>
                                {if let (Some(high), Some(low)) = (f.high, f.low) {
                                    html! {
                                        <div class="fs-6">
                                            {format!("{}°C / {}°C", high, low)}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                                {if let Some(pop) = f.pop {
                                    if pop > 50 {
                                        html! {
                                            <div class="fs-6 text-warning">
                                                {"⚠️ "}{format!("{}% rain", pop)}
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct BinImageProps {
    pub src: AttrValue,
    pub alt: AttrValue,
    // NEW: Optional property to inject custom size/style when necessary
    #[prop_or_default]
    pub size_style: AttrValue, 
}

#[function_component]
fn BinImage(&BinImageProps { ref src, ref alt, ref size_style }: &BinImageProps) -> Html {
    
    // Base style that applies to all bins
    let base_style = "object-fit: contain; margin-right: 5px; border: none; outline: none; box-shadow: none; background: transparent; padding: 0; display: inline-block; vertical-align: middle;";

    // Determine the final style. All images should have a height of 4rem by default.
    let final_style = if size_style.is_empty() {
        // Default style for images that don't need a specific width (Green, Garbage, Blue, Christmas Tree)
        AttrValue::from(format!("height: 4rem; {}", base_style))
    } else {
        // Custom style for the Yard Waste image to fix its aspect ratio
        AttrValue::from(format!("{} {}", size_style, base_style))
    };


    html! {
        <img 
            class="bin-icon"
            src={src.clone()} 
            alt={alt.clone()} 
            style={final_style} // Use the calculated style
        />
    }
}

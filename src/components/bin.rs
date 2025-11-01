use chrono::prelude::*;
use chrono::{DateTime, Local, Weekday};
use futures::StreamExt;
use std::time::Duration;
use yew::platform::time::interval;
use yew::{function_component, html, AttrValue, Component, Context, Html, Properties};

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

pub struct BinComponent {
    current_time: DateTime<Local>,
}

pub enum BinComponentMsg {
    ClockTicked(DateTime<Local>),
}

impl Component for BinComponent {
    type Message = BinComponentMsg;
    type Properties = ();

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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let show_brown_bin = is_yard_waste_season();
        let show_christmas_tree = is_christmas_tree_season();
        
        // Calculate days until pickup
        let days_until_pickup = (3 + 7 - self.current_time.weekday().num_days_from_monday()) % 7;
        let days_text = if days_until_pickup == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", days_until_pickup)
        };
        
        html! {
            <div class="d-flex align-items-center">
                // Only Green bin is always displayed
                <BinImage src="./images/GreenBin.png" alt="Green Bin" />

                // Alternating Blue vs Black and Brown bins
                {
                    match get_alternate_bin() {
                        BinVariation::Yellow => html! { 
                            <> 
                                <BinImage src="./images/GarbageBin.png" alt="Garbage Bin" />
                                // Brown bin only shown during yard waste season
                                if show_brown_bin {
                                    <BinImage src="./images/YardWaste.png" alt="Yard Waste" />
                                }
                            </> 
                        },
                        BinVariation::None => html! { <BinImage src="./images/BlueBin.png" alt="Blue Bin" /> }
                    }
                }

                // Christmas tree icon during collection period
                if show_christmas_tree {
                    <BinImage src="./images/Christmastree.png" alt="Christmas Tree" />
                }

                <div class="fs-1 fw-bold text-white">
                    if self.current_time.weekday() == Weekday::Thu {
                        {"BIN DAY TODAY!!"}
                    } else {
                        {days_text}
                    }
                </div>
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct BinImageProps {
    pub src: AttrValue,
    pub alt: AttrValue,
}

#[function_component]
fn BinImage(&BinImageProps { ref src, ref alt }: &BinImageProps) -> Html {
    html! {
        <img 
            src={src.clone()} 
            alt={alt.clone()} 
            style="height: 80px; width: auto; margin-right: 5px;"
        />
    }
}

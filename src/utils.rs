use gloo_console::log;
use gloo_net::http::Request;
use serde::de::DeserializeOwned;

pub async fn fetch<T>(url: String) -> T
where
    T: DeserializeOwned + Default,
{
    let resp = Request::get(&url).send().await;
    match resp {
        Ok(r) => {
            if r.ok() {
                match r.json::<T>().await {
                    Ok(data) => return data,
                    Err(e) => {
                        log!(format!("JSON parse error: {:?}", e));
                    }
                }
            } else {
                log!(format!("HTTP error: {} {}", r.status(), r.status_text()));
            }
        }
        Err(e) => {
            log!(format!("Fetch failed: {:?}", e));
        }
    }
    T::default()
}

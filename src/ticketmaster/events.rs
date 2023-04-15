use std::{collections::HashMap, error::Error};

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::value::Value;

use super::{shared::TicketMasterError, API_PREFIX};

const EVENT_ENDPOINT: &str = "events.json?";

#[derive(Debug)]
pub struct EventParams {
    api_key: String,
    location: Option<String>,
    search_terms: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventImage {
    link: String,
    width: Option<u32>,
    height: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventLocation {
    area_name: Option<String>,
    address_line_1: Option<String>,
    address_line_2: Option<String>,
    address_line_3: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: (Option<String>, Option<String>),
    postal_code: Option<String>,
    name: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    name: String,
    id: String,
    url: String,
    images: Vec<EventImage>,
    description: Option<String>,
    additional_info: Option<String>,
    start: Option<String>,
    end: Option<String>,
    info: Option<String>,
    please_note: Option<String>,
    location: Option<EventLocation>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResult {
    next_page: Option<String>,
    prev_page: Option<String>,
    events: Vec<Event>
}

impl EventParams {
    pub fn new(api_key: String, search_terms: Option<String>, location: Option<String>) -> Result<EventParams, TicketMasterError> {
        if search_terms.is_none() && location.is_none() {
            return Err(TicketMasterError::new(None));
        }

        return Ok(EventParams {
            api_key,
            search_terms,
            location
        });
    }
}

pub async fn get_events(client: &Client, args: EventParams) -> Result<EventResult, Box<dyn Error>> {
    let mut endpoint = API_PREFIX.to_string();
    let mut has_argument = false;
    match args.search_terms {
        Some(search) => {
            endpoint.push_str(&search);
            has_argument = true;
        },
        _ => ()
    };

    match args.location {
        Some(location) => {
            if(has_argument) {
                endpoint.push_str("&")
            }
            endpoint.push_str(&location);
            has_argument = true;
        },
        _ => ()
    }

    tracing::info!("Making request to {}", endpoint);

    let response = client
        .get(endpoint)
        .send()
        .await?
        .json::<HashMap<String, Value>>()
        .await?;

    match parse_event_response(response) {
        Ok(event_result) => Ok(event_result),
        Err(err) => Err(Box::new(err))
    }
}

const EVENT_PARSE_ERROR: &str = "Failed to parse response from TicketMaster API";

fn parse_event_response(event: HashMap<String, Value>) -> Result<EventResult, TicketMasterError> {
    // Todo: Make the unwraps safer.
    
    let links = event.get("_links");
    let (next_page, prev_page) = parse_links(links);

    let embedded_container = event.get("_embedded");
    if embedded_container.is_none() {
        return Err(TicketMasterError::new(Some(EVENT_PARSE_ERROR.to_string())));
    }

    let events_array_val = embedded_container.unwrap().as_object().unwrap().get("events");
    if events_array_val.is_none() {
        return Err(TicketMasterError::new(Some(EVENT_PARSE_ERROR.to_string())));
    }

    let events_array = events_array_val.unwrap().as_array().unwrap();

    let events = parse_events_array(events_array);

    return Ok(EventResult { next_page, prev_page, events })
}

fn parse_links(links: Option<&Value>) -> (Option<String>, Option<String>) {
    let mut next_page: Option<String> = None;
    let mut prev_page: Option<String> = None;

    if links.is_some() {
        let links_obj = links.unwrap().as_object();
        if links_obj.is_some() {
            let next_obj = links_obj.unwrap().get("next");
            let prev_obj = links_obj.unwrap().get("prev");

            if next_obj.is_some() {
                let next_href = next_obj
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .get("href");

                if next_href.is_some() {
                    next_page = Some(next_href.unwrap().as_str().unwrap().to_string());
                }
            }

            if prev_obj.is_some() {
                let prev_href = prev_obj
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .get("href");

                if prev_href.is_some() {
                    prev_page = Some(prev_href.unwrap().as_str().unwrap().to_string());
                }
            }
        }
    }

    (next_page, prev_page)
}

fn parse_events_array(events_array: &Vec<Value>) -> Vec<Event> {
    let mut result: Vec<Event> = Vec::new();

    for event in events_array {
        let name = event.get("name").unwrap().as_str().unwrap().to_string();
        let id = event.get("id*").unwrap().as_str().unwrap().to_string();
        let url = event.get("url").unwrap().as_str().unwrap().to_string();
        let description = match event.get("description") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            None => None
        };
        let additional_info = match event.get("additionalInfo") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            None => None
        };

        let start = match event.get("dates") {
            Some(dates) => {
                match dates.get("start") {
                    Some(start) => {
                        let date_time = start.get("dateTime");
                        let date = start.get("localDate");
                        if date_time.is_some() {
                            Some(date_time.unwrap().as_str().unwrap().to_string())
                        } else if date.is_some() {
                            Some(date.unwrap().as_str().unwrap().to_string())
                        } else {
                            None
                        }
                    }
                    None => None
                }
            }
            None => None
        };

        let end = match event.get("dates") {
            Some(dates) => {
                match dates.get("end") {
                    Some(end) => {
                        let date_time = end.get("dateTime");
                        let date = end.get("localDate");
                        if date_time.is_some() {
                            Some(date_time.unwrap().as_str().unwrap().to_string())
                        } else if date.is_some() {
                            Some(date.unwrap().as_str().unwrap().to_string())
                        } else {
                            None
                        }
                    }
                    None => None
                }
            }
            None => None
        };

        let info = match event.get("info") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            None => None
        };

        let please_note = match event.get("pleaseNote") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            None => None
        };

        let location = parse_event_location(event);
        
        let images = parse_event_images(event);

        result.push(Event { name, id, url, images, description, additional_info, start, end, info, please_note, location })
    }

    result
}

fn parse_event_location(event: &Value) -> Option<EventLocation> {
    let place_val = event.get("place");
    if place_val.is_none() {
        return None;
    }

    let place = place_val.unwrap();

    let area = match place.get("area") {
        Some(obj) => {
            match obj.get("name") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            }
        },
        None => None
    };

    let mut line1: Option<String> = None;
    let mut line2: Option<String> = None;
    let mut line3: Option<String> = None;

    match place.get("address")  {
        Some(inner) => {
            line1 = match inner.get("line1") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            };
            line2 = match inner.get("line2") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            };
            line3 = match inner.get("line3") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            };
        }
        None => ()
    }

    let city = match place.get("city") {
        Some(inner) => {
            match inner.get("name") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            }
        }
        None => None
    };

    let state = match place.get("state") {
        Some(inner) => {
            match inner.get("name") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            }
        }
        None => None
    };

    let mut country: Option<String> = None;
    let mut country_code: Option<String> = None;

    match place.get("country") {
        Some(inner) => {
            country = match inner.get("name") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            };
            country_code = match inner.get("countryCode") {
                Some(text) => Some(text.as_str().unwrap().to_string()),
                None => None
            };
        },
        None => ()
    }

    let postal_code = match place.get("postal_code") {
        Some(text) => Some(text.as_str().unwrap().to_string()),
        None => None
    };

    let name = match place.get("name") {
        Some(text) => Some(text.as_str().unwrap().to_string()),
        None => None
    };

    Some(EventLocation {
        area_name: area,
        address_line_1: line1,
        address_line_2: line2,
        address_line_3: line3,
        city,
        state,
        country: (country, country_code),
        postal_code,
        name
    })
}

fn parse_event_images(event: &Value) -> Vec<EventImage> {
    let mut images: Vec<EventImage> = Vec::new();

    let image_val = event.get("images");
    if image_val.is_none() {
        return images;
    }

    let images_array = image_val.unwrap().as_array().unwrap();
    for image in images_array {
        let url = match image.get("url") {
            Some(text) => Some(text.as_str().unwrap().to_string()),
            None => None
        };

        if url.is_none() {
            continue;
        }
        
        let width = match image.get("width") {
            Some(text) => Some(text.as_i64().unwrap() as u32),
            None => None
        };

        let height = match image.get("height") {
            Some(text) => Some(text.as_i64().unwrap() as u32),
            None => None
        };

        images.push(EventImage { link: url.unwrap(), width, height });
    }

    images
}
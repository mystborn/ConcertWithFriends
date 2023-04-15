use std::{error::Error, fmt};

pub const API_PREFIX: &str = "https://app.ticketmaster.com/discovery/v2/";

#[derive(Debug)]
pub struct TicketMasterError {
    message: Option<String>
}

impl fmt::Display for TicketMasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(msg) => write!(f, "{}", msg),
            None => write!(f, "Encountered an error with the ticketmaster api")
        }
    }
}

impl Error for TicketMasterError {}

impl TicketMasterError {
    pub fn new(message: Option<String>) -> TicketMasterError {
        TicketMasterError {
            message
        }
    }
}
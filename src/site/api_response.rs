use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse<T> {
    pub success: bool,
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIError {
    //Will be None if 200-290
    pub status_code: Option<u16>,
    //User friendly messages will be provided for some cases
    pub user_friendly_message: Option<String>,
    //Look into that specific API for what this will be set to. This is something that specific api will control
    pub error_code: Option<String>,
}

impl<T: Serialize> APIResponse<T> {
    pub fn new(success: bool, data: Option<T>) -> APIResponse<T> {
        return APIResponse { success, data };
    }
}

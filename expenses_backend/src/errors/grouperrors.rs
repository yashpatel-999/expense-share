use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum GroupError {
    NotFound,
    NotAMember,
    AlreadyMember,
    InsufficientPermissions,
    GroupFull,
}

impl fmt::Display for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupError::NotFound => write!(f, "Group not found"),
            GroupError::NotAMember => write!(f, "User is not a member of this group"),
            GroupError::AlreadyMember => write!(f, "User is already a member of this group"),
            GroupError::InsufficientPermissions => {
                write!(f, "Insufficient permissions for this group operation")
            }
            GroupError::GroupFull => write!(f, "Group has reached maximum capacity"),
        }
    }
}

impl ResponseError for GroupError {
    fn status_code(&self) -> StatusCode {
        match self {
            GroupError::NotFound => StatusCode::NOT_FOUND,
            GroupError::NotAMember | GroupError::InsufficientPermissions => StatusCode::FORBIDDEN,
            GroupError::AlreadyMember | GroupError::GroupFull => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": "group_error",
            "message": self.to_string(),
        }))
    }
}

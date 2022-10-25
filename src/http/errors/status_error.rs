use std::{error::Error as StdError, fmt::Display};

use async_trait::async_trait;
use hyper::StatusCode;

use crate::writer::Writer;

pub type StatusResult<T> = Result<T, StatusError>;

macro_rules! default_errors {
    ($($sname:ident, $code:expr, $name:expr, $summary:expr);+) => {
        $(
            pub fn $sname() -> StatusError {
                StatusError {
                    code: $code,
                    name: $name.into(),
                    summary: Some($summary.into()),
                    detail: None,
                }
            }
        )+
    };
}
// macro_rules! default_errors_reverse {
//     ($($tname:expr, $rname:ident);+) => {
//         $(
//             $tname => Some($rname()),
//         )+
//     };
// }
#[derive(Debug)]
pub struct StatusError {
    pub code: StatusCode,
    pub name: String,
    pub summary: Option<String>,
    pub detail: Option<String>,
}

impl StatusError {
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
    default_errors! {
    bad_request,                        StatusCode::BAD_REQUEST,            "Bad Request", "The request could not be understood by the server due to malformed syntax.";
    unauthorized,                       StatusCode::UNAUTHORIZED,           "Unauthorized", "The request requires user authentication.";
    payment_required,                   StatusCode::PAYMENT_REQUIRED,       "Payment Required", "The request could not be processed due to lack of payment.";
    forbidden,                          StatusCode::FORBIDDEN,              "Forbidden", "The server refused to authorize the request.";
    not_found,                          StatusCode::NOT_FOUND,              "Not Found", "The requested resource could not be found.";
    method_not_allowed,                 StatusCode::METHOD_NOT_ALLOWED,     "Method Not Allowed", "The request method is not supported for the requested resource.";
    not_acceptable,                     StatusCode::NOT_ACCEPTABLE,         "Not Acceptable", "The requested resource is capable of generating only content not acceptable according to the Accept headers sent in the request.";
    proxy_authentication_required,      StatusCode::PROXY_AUTHENTICATION_REQUIRED,  "Proxy Authentication Required", "Authentication with the proxy is required.";
    request_timeout,                    StatusCode::REQUEST_TIMEOUT,        "Request Timeout", "The server timed out waiting for the request.";
    conflict,                           StatusCode::CONFLICT,               "Conflict", "The request could not be processed because of a conflict in the request.";
    gone,                               StatusCode::GONE,                   "Gone", "The resource requested is no longer available and will not be available again.";
    length_required,                    StatusCode::LENGTH_REQUIRED,        "Length Required", "The request did not specify the length of its content, which is required by the requested resource.";
    precondition_failed,                StatusCode::PRECONDITION_FAILED,    "Precondition Failed", "The server does not meet one of the preconditions specified in the request.";
    payload_too_large,                  StatusCode::PAYLOAD_TOO_LARGE,      "Payload Too Large", "The request is larger than the server is willing or able to process.";
    uri_too_long,                       StatusCode::URI_TOO_LONG,           "URI Too Long", "The URI provided was too long for the server to process.";
    unsupported_media_type,             StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type", "The request entity has a media type which the server or resource does not support.";
    range_not_satisfiable,              StatusCode::RANGE_NOT_SATISFIABLE,  "Range Not Satisfiable", "The portion of the requested file cannot be supplied by the server.";
    expectation_failed,                 StatusCode::EXPECTATION_FAILED,     "Expectation Failed", "The server cannot meet the requirements of the expect request-header field.";
    im_a_teapot,                        StatusCode::IM_A_TEAPOT,            "I'm a teapot", "I was requested to brew coffee, and I am a teapot.";
    misdirected_request,                StatusCode::MISDIRECTED_REQUEST,    "Misdirected Request", "The server cannot produce a response for this request.";
    unprocessable_entity,               StatusCode::UNPROCESSABLE_ENTITY,   "Unprocessable Entity", "The request was well-formed but was unable to be followed due to semantic errors.";
    locked,                             StatusCode::LOCKED,                 "Locked", "The source or destination resource of a method is locked.";
    failed_dependency,                  StatusCode::FAILED_DEPENDENCY,      "Failed Dependency", "The method could not be performed on the resource because the requested action depended on another action and that action failed.";
    upgrade_required,                   StatusCode::UPGRADE_REQUIRED,       "Upgrade Required", "Switching to the protocol in the Upgrade header field is required.";
    precondition_required,              StatusCode::PRECONDITION_REQUIRED,  "Precondition Required", "The server requires the request to be conditional.";
    too_many_requests,                  StatusCode::TOO_MANY_REQUESTS,      "Too Many Requests", "Too many requests have been received recently.";
    request_header_fields_toolarge,     StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,    "Request Header Fields Too Large", "The server is unwilling to process the request because either  an individual header field, or all the header fields collectively, are too large.";
    unavailable_for_legalreasons,       StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,      "Unavailable For Legal Reasons", "The requested resource is unavailable due to a legal demand to deny access to this resource.";
    internal_server_error,              StatusCode::INTERNAL_SERVER_ERROR,  "Internal Server Error", "The server encountered an internal error while processing this request.";
    not_implemented,                    StatusCode::NOT_IMPLEMENTED,        "Not Implemented", "The server either does not recognize the request method, or it lacks the ability to fulfill the request.";
    bad_gateway,                        StatusCode::BAD_GATEWAY,            "Bad Gateway", "Received an invalid response from an inbound server it accessed while attempting to fulfill the request.";
    service_unavailable,                StatusCode::SERVICE_UNAVAILABLE,    "Service Unavailable", "The server is currently unavailable.";
    gateway_timeout,                    StatusCode::GATEWAY_TIMEOUT,        "Gateway Timeout", "The server did not receive a timely response from an upstream server.";
    http_version_not_supported,         StatusCode::HTTP_VERSION_NOT_SUPPORTED, "HTTP Version Not Supported", "The server does not support, or refuses to support, the major version of HTTP that was used in the request message.";
    variant_also_negotiates,            StatusCode::VARIANT_ALSO_NEGOTIATES, "Variant Also Negotiates", "The server has an internal configuration error.";
    insufficient_storage,               StatusCode::INSUFFICIENT_STORAGE,    "Insufficient Storage", "The method could not be performed on the resource because the server is unable to store the representation needed to successfully complete the request.";
    loop_detected,                      StatusCode::LOOP_DETECTED,           "Loop Detected", "the server terminated an operation because it encountered an infinite loop while processing a request with \"Depth: infinity\".";
    not_extended,                       StatusCode::NOT_EXTENDED,            "Not Extended", "Further extensions to the request are required for the server to fulfill it.";
    network_authentication_required,    StatusCode::NETWORK_AUTHENTICATION_REQUIRED, "Network Authentication Required", "the client needs to authenticate to gain network access."
    }
}

impl StdError for StatusError {}

impl Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "code: {}", &self.code)?;
        write!(f, "name: {}", &self.name)?;
        write!(f, "summary: {:?}", &self.summary)?;
        write!(f, "detail: {:?}", &self.detail)?;
        Ok(())
    }
}

impl StatusError {
    pub fn from_code(code: StatusCode) -> Option<StatusError> {
        match code {
            StatusCode::BAD_REQUEST => Some(StatusError::bad_request()),
            StatusCode::UNAUTHORIZED => Some(StatusError::unauthorized()),
            StatusCode::PAYMENT_REQUIRED => Some(StatusError::payment_required()),
            StatusCode::FORBIDDEN => Some(StatusError::forbidden()),
            StatusCode::NOT_FOUND => Some(StatusError::not_found()),
            StatusCode::METHOD_NOT_ALLOWED => Some(StatusError::method_not_allowed()),
            StatusCode::NOT_ACCEPTABLE => Some(StatusError::not_acceptable()),
            StatusCode::PROXY_AUTHENTICATION_REQUIRED => Some(StatusError::proxy_authentication_required()),
            StatusCode::REQUEST_TIMEOUT => Some(StatusError::request_timeout()),
            StatusCode::CONFLICT => Some(StatusError::conflict()),
            StatusCode::GONE => Some(StatusError::gone()),
            StatusCode::LENGTH_REQUIRED => Some(StatusError::length_required()),
            StatusCode::PRECONDITION_FAILED => Some(StatusError::precondition_failed()),
            StatusCode::PAYLOAD_TOO_LARGE => Some(StatusError::payload_too_large()),
            StatusCode::URI_TOO_LONG => Some(StatusError::uri_too_long()),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => Some(StatusError::unsupported_media_type()),
            StatusCode::RANGE_NOT_SATISFIABLE => Some(StatusError::range_not_satisfiable()),
            StatusCode::EXPECTATION_FAILED => Some(StatusError::expectation_failed()),
            StatusCode::IM_A_TEAPOT => Some(StatusError::im_a_teapot()),
            StatusCode::MISDIRECTED_REQUEST => Some(StatusError::misdirected_request()),
            StatusCode::UNPROCESSABLE_ENTITY => Some(StatusError::unprocessable_entity()),
            StatusCode::LOCKED => Some(StatusError::locked()),
            StatusCode::FAILED_DEPENDENCY => Some(StatusError::failed_dependency()),
            StatusCode::UPGRADE_REQUIRED => Some(StatusError::upgrade_required()),
            StatusCode::PRECONDITION_REQUIRED => Some(StatusError::precondition_required()),
            StatusCode::TOO_MANY_REQUESTS => Some(StatusError::too_many_requests()),
            StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => Some(StatusError::request_header_fields_toolarge()),
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => Some(StatusError::unavailable_for_legalreasons()),
            StatusCode::INTERNAL_SERVER_ERROR => Some(StatusError::internal_server_error()),
            StatusCode::NOT_IMPLEMENTED => Some(StatusError::not_implemented()),
            StatusCode::BAD_GATEWAY => Some(StatusError::bad_gateway()),
            StatusCode::SERVICE_UNAVAILABLE => Some(StatusError::service_unavailable()),
            StatusCode::GATEWAY_TIMEOUT => Some(StatusError::gateway_timeout()),
            StatusCode::HTTP_VERSION_NOT_SUPPORTED => Some(StatusError::http_version_not_supported()),
            StatusCode::VARIANT_ALSO_NEGOTIATES => Some(StatusError::variant_also_negotiates()),
            StatusCode::INSUFFICIENT_STORAGE => Some(StatusError::insufficient_storage()),
            StatusCode::LOOP_DETECTED => Some(StatusError::loop_detected()),
            StatusCode::NOT_EXTENDED => Some(StatusError::not_extended()),
            StatusCode::NETWORK_AUTHENTICATION_REQUIRED => Some(StatusError::network_authentication_required()),
            _ => None,
        }
    }
}

// TODO: impl Writer for StatusError
#[async_trait]
impl Writer for StatusError {
    async fn write(mut self) {}
}

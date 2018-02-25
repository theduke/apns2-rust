use failure::Error;

/// The reason for a failure returned by the APN api.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ApiErrorReason {
    BadCollapseId,
    BadDeviceToken,
    BadExpirationDate,
    BadMessageId,
    BadPriority,
    BadTopic,
    DeviceTokenNotForTopic,
    DuplicateHeaders,
    IdleTimeout,
    MissingDeviceToken,
    MissingTopic,
    PayloadEmpty,
    TopicDisallowed,
    BadCertificate,
    BadCertificateEnvironment,
    ExpiredProviderToken,
    Forbidden,
    InvalidProviderToken,
    MissingProviderToken,
    BadPath,
    MethodNotAllowed,
    Unregistered,
    PayloadTooLarge,
    TooManyProviderTokenUpdates,
    TooManyRequests,
    InternalServerError,
    ServiceUnavailable,
    Shutdown,
    Other(String),
}

impl ApiErrorReason {
    fn from_str(value: &str) -> Self {
        use self::ApiErrorReason::*;
        match value {
            "BadCollapseId" => BadCollapseId,
            "BadDeviceToken" => BadDeviceToken,
            "BadExpirationDate" => BadExpirationDate,
            "BadMessageId" => BadMessageId,
            "BadPriority" => BadPriority,
            "BadTopic" => BadTopic,
            "DeviceTokenNotForTopic" => DeviceTokenNotForTopic,
            "DuplicateHeaders" => DuplicateHeaders,
            "IdleTimeout" => IdleTimeout,
            "MissingDeviceToken" => MissingDeviceToken,
            "MissingTopic" => MissingTopic,
            "PayloadEmpty" => PayloadEmpty,
            "TopicDisallowed" => TopicDisallowed,
            "BadCertificate" => BadCertificate,
            "BadCertificateEnvironment" => BadCertificateEnvironment,
            "ExpiredProviderToken" => ExpiredProviderToken,
            "Forbidden" => Forbidden,
            "InvalidProviderToken" => InvalidProviderToken,
            "MissingProviderToken" => MissingProviderToken,
            "BadPath" => BadPath,
            "MethodNotAllowed" => MethodNotAllowed,
            "Unregistered" => Unregistered,
            "PayloadTooLarge" => PayloadTooLarge,
            "TooManyProviderTokenUpdates" => TooManyProviderTokenUpdates,
            "TooManyRequests" => TooManyRequests,
            "InternalServerError" => InternalServerError,
            "ServiceUnavailable" => ServiceUnavailable,
            "Shutdown" => Shutdown,
            x => Other(x.to_string()),
        }
    }

    fn to_str(&self) -> &str {
        use self::ApiErrorReason::*;
        match self {
            &BadCollapseId => "BadCollapseId",
            &BadDeviceToken => "BadDeviceToken",
            &BadExpirationDate => "BadExpirationDate",
            &BadMessageId => "BadMessageId",
            &BadPriority => "BadPriority",
            &BadTopic => "BadTopic",
            &DeviceTokenNotForTopic => "DeviceTokenNotForTopic",
            &DuplicateHeaders => "DuplicateHeaders",
            &IdleTimeout => "IdleTimeout",
            &MissingDeviceToken => "MissingDeviceToken",
            &MissingTopic => "MissingTopic",
            &PayloadEmpty => "PayloadEmpty",
            &TopicDisallowed => "TopicDisallowed",
            &BadCertificate => "BadCertificate",
            &BadCertificateEnvironment => "BadCertificateEnvironment",
            &ExpiredProviderToken => "ExpiredProviderToken",
            &Forbidden => "Forbidden",
            &InvalidProviderToken => "InvalidProviderToken",
            &MissingProviderToken => "MissingProviderToken",
            &BadPath => "BadPath",
            &MethodNotAllowed => "MethodNotAllowed",
            &Unregistered => "Unregistered",
            &PayloadTooLarge => "PayloadTooLarge",
            &TooManyProviderTokenUpdates => "TooManyProviderTokenUpdates",
            &TooManyRequests => "TooManyRequests",
            &InternalServerError => "InternalServerError",
            &ServiceUnavailable => "ServiceUnavailable",
            &Shutdown => "Shutdown",
            &Other(ref val) => val,
        }
    }

    pub fn is_bad_device_token(&self) -> bool {
        match self {
            &ApiErrorReason::BadDeviceToken => true,
            _ => false,
        }
    }
}

impl ::std::fmt::Display for ApiErrorReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// Error returned by the APN api.
#[derive(Fail, Serialize, Deserialize, Clone, Debug)]
#[fail(display = "{} (status: {}", reason, status)]
pub struct ApiError {
    pub status: u32,
    pub reason: ApiErrorReason,
}

impl ApiError {
    pub fn is_bad_device_token(&self) -> bool {
        return self.reason.is_bad_device_token();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ErrorResponse {
    pub reason: String,
}

impl ErrorResponse {
    pub fn parse_payload(data: &[u8]) -> ApiErrorReason {
        match ::serde_json::from_slice::<ErrorResponse>(data) {
            Ok(response) => ApiErrorReason::from_str(&response.reason),
            Err(_) => {
                let msg = format!("Unknown API response: {:?}", data);
                ApiErrorReason::Other(msg)
            }
        }
    }
}

#[derive(Fail, Debug)]
pub enum SendError {
    #[fail(display = "{}", _0)]
    Api(ApiError),
    #[fail(display = "{}", _0)]
    Other(Error),
}

impl SendError {
    pub fn as_api_error(&self) -> Option<&ApiError> {
        match self {
            &SendError::Api(ref e) => Some(e),
            _ => None,
        }
    }

    pub fn is_bad_device_token(&self) -> bool {
        match self {
            &SendError::Api(ref e) => e.is_bad_device_token(),
            _ => false,
        }
    }
}

impl From<::curl::Error> for SendError {
    fn from(e: ::curl::Error) -> Self {
        SendError::Other(e.into())
    }
}

impl From<::serde_json::Error> for SendError {
    fn from(e: ::serde_json::Error) -> Self {
        SendError::Other(e.into())
    }
}

impl From<ApiError> for SendError {
    fn from(e: ApiError) -> Self {
        SendError::Api(e)
    }
}

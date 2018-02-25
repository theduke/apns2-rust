use uuid::Uuid;

/// APNS production endpoint.
pub static APN_URL_PRODUCTION: &'static str = "https://api.push.apple.com";

/// APNS development endpoint.
pub static APN_URL_DEV: &'static str = "https://api.development.push.apple.com";

/// Notification priority.
/// See APNS documentation for the effects.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Priority {
    #[serde(rename = "5")]
    Low, // 5
    #[serde(rename = "10")]
    High, // 10
}

impl Priority {
    /// Convert Priority to it's numeric value.
    pub fn to_int(self) -> u32 {
        match self {
            Priority::Low => 5,
            Priority::High => 10,
        }
    }
}

#[derive(Fail, Debug)]
#[fail(display = "CollapseId too long (must be at most 64 bytes)")]
pub struct CollapseIdTooLongError;

/// Wrapper type for collapse ids.
/// It may be an arbitrary string, but is limited in length to at most 63 bytes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CollapseId(String);

impl CollapseId {
    /// Construct a new collapse id.
    /// Returns an error if id exceeds the maximum length of 64 bytes.
    pub fn new(value: String) -> Result<Self, CollapseIdTooLongError> {
        // CollapseID must be at most 64 bytes long.
        if value.len() > 64 {
            Err(CollapseIdTooLongError)
        } else {
            Ok(CollapseId(value))
        }
    }

    /// Get id as a raw str.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Alert content for a notification.
///
/// See the official documentation for details:
/// https://developer.apple.com/library/content/documentation/NetworkingInternet/Conceptual/RemoteNotificationsPG/PayloadKeyReference.html
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct AlertPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(rename = "title-loc-key", skip_serializing_if = "Option::is_none")]
    pub title_loc_key: Option<String>,
    #[serde(rename = "title-loc-args", skip_serializing_if = "Option::is_none")]
    pub title_loc_args: Option<Vec<String>>,
    #[serde(rename = "action-loc-key", skip_serializing_if = "Option::is_none")]
    pub action_loc_key: Option<String>,
    #[serde(rename = "loc-key", skip_serializing_if = "Option::is_none")]
    pub loc_key: Option<String>,
    #[serde(rename = "loc-args", skip_serializing_if = "Option::is_none")]
    pub loc_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_image: Option<String>,
}

impl AlertPayload {
    fn new(title: Option<String>, body: Option<String>) -> Self {
        AlertPayload {
            title: title,
            body: body,
            title_loc_key: None,
            title_loc_args: None,
            action_loc_key: None,
            loc_key: None,
            loc_args: None,
            loc_image: None,
        }
    }
}

/// The alert content.
/// This can either be a plain message string, or an AlertPayload with more
/// configuration.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Alert {
    Simple(String),
    Payload(AlertPayload),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Payload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
    /// Updates the numeric badge for the app. Set to 0 to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    /// Sound to play. Use 'default' for the default sound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    /// Set to true to mark the app as having content available.
    #[serde(rename = "content-available", skip_serializing_if = "Option::is_none")]
    pub content_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(rename = "thread-id", skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// A full json request object for sending a notification to the API.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ApnsRequest {
    pub aps: Payload,
}

/// A notification struct contains all relevant data for a notification request
/// sent to the APNS API.
/// This includes other options not contained in the payload.
/// These options are transferred with HTTP request headers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    /// The topic to use. Usually the app bundle id.
    pub topic: String,
    pub device_token: String,
    pub payload: Payload,

    /// Optional id identifying the message.
    pub id: Option<Uuid>,
    /// Optional expiration time as UNIX timestamp.
    pub expiration: Option<u64>,
    /// Priority for the notification.
    pub priority: Option<Priority>,
    pub collapse_id: Option<CollapseId>,
}

impl Notification {
    /// Create a new notification.
    pub fn new(topic: String, device_token: String, payload: Payload) -> Self {
        Notification {
            topic,
            device_token,
            payload,
            id: None,
            expiration: None,
            priority: None,
            collapse_id: None,
        }
    }
}

/// A builder for convenient construction of notifications.
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new(topic: String, device_id: String) -> Self {
        NotificationBuilder {
            notification: Notification::new(topic, device_id, Payload::default()),
        }
    }

    pub fn payload(mut self, payload: Payload) -> Self {
        self.notification.payload = payload;
        self
    }

    pub fn alert<S: Into<String>>(mut self, alert: S) -> Self {
        self.notification.payload.alert = Some(Alert::Simple(alert.into()));
        self
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        let title = title.into();
        let payload = match self.notification.payload.alert.take() {
            None => AlertPayload::new(Some(title), None),
            Some(Alert::Simple(_)) => AlertPayload::new(Some(title), None),
            Some(Alert::Payload(mut payload)) => {
                payload.title = Some(title);
                payload
            }
        };
        self.notification.payload.alert = Some(Alert::Payload(payload));
        self
    }

    pub fn body<S: Into<String>>(mut self, body: S) -> Self {
        let body = body.into();
        let payload = match self.notification.payload.alert.take() {
            None => AlertPayload::new(None, Some(body)),
            Some(Alert::Simple(title)) => AlertPayload::new(Some(title), Some(body)),
            Some(Alert::Payload(mut payload)) => {
                payload.body = Some(body);
                payload
            }
        };
        self.notification.payload.alert = Some(Alert::Payload(payload));
        self
    }

    pub fn badge(mut self, number: u32) -> Self {
        self.notification.payload.badge = Some(number);
        self
    }

    pub fn sound<S: Into<String>>(mut self, sound: S) -> Self {
        self.notification.payload.sound = Some(sound.into());
        self
    }

    pub fn content_available(mut self) -> Self {
        self.notification.payload.content_available = Some(true);
        self
    }

    pub fn category(mut self, category: String) -> Self {
        self.notification.payload.category = Some(category);
        self
    }

    pub fn thread_id(mut self, thread_id: String) -> Self {
        self.notification.payload.thread_id = Some(thread_id);
        self
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.notification.id = Some(id);
        self
    }

    pub fn expiration(mut self, expiration: u64) -> Self {
        self.notification.expiration = Some(expiration);
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.notification.priority = Some(priority);
        self
    }

    pub fn collapse_id(mut self, id: CollapseId) -> Self {
        self.notification.collapse_id = Some(id);
        self
    }

    pub fn build(self) -> Notification {
        self.notification
    }
}

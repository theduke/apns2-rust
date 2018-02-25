#![deny(warnings)]

extern crate curl;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

mod types;
pub use self::types::*;

mod error;
use self::error::*;

use std::path::{Path, PathBuf};
use std::cell::RefCell;

use uuid::Uuid;
use failure::Error;
use curl::easy::{Easy2, Handler, HttpVersion, List, WriteError};

/// Writer used by curl.
struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

#[derive(Clone, Debug)]
pub struct ProviderCertificate {
    pub p12_path: PathBuf,
    pub passphrase: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Auth {
    ProviderCertificate(ProviderCertificate),
}

impl Auth {
    fn as_cert(&self) -> &ProviderCertificate {
        match self {
            &Auth::ProviderCertificate(ref c) => c,
        }
    }
}

pub struct ApnsSync {
    production: bool,
    verbose: bool,
    delivery_disabled: bool,
    auth: Auth,
    easy: RefCell<Easy2<Collector>>,
}

impl ApnsSync {
    pub fn new(auth: Auth) -> Result<Self, Error> {
        let mut easy = Easy2::new(Collector(Vec::new()));

        easy.http_version(HttpVersion::V2)?;
        // easy.connect_only(true)?;
        // easy.url(APN_URL_PRODUCTION)?;

        // Configure curl for client certificate.
        {
            let cert = auth.as_cert();

            easy.ssl_cert(&cert.p12_path)?;
            if let Some(ref pw) = cert.passphrase.as_ref() {
                easy.key_password(&pw)?;
            }
        }

        let apns = ApnsSync {
            production: true,
            verbose: false,
            delivery_disabled: false,
            auth,
            easy: RefCell::new(easy),
        };
        Ok(apns)
    }

    pub fn with_certificate<P: AsRef<Path>>(
        path: P,
        passphrase: Option<String>,
    ) -> Result<ApnsSync, Error> {
        Self::new(Auth::ProviderCertificate(ProviderCertificate {
            p12_path: path.as_ref().to_path_buf(),
            passphrase,
        }))
    }

    /// Enable/disable verbose debug logging to stderr.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Set API endpoint to use (production or development sandbox).
    pub fn set_production(&mut self, production: bool) {
        self.production = production;
    }

    /// *ATTENTION*: This completely disables actual communication with the
    /// APNS api.
    ///
    /// No connection will be established.
    ///
    /// Useful for integration tests in a larger application when nothing should
    /// actually be sent.
    pub fn disable_delivery_for_testing(&mut self) {
        self.delivery_disabled = true;
    }

    /// Build the url for a device token.
    fn build_url(&self, device_token: &str) -> String {
        let root = if self.production {
            APN_URL_PRODUCTION
        } else {
            APN_URL_DEV
        };
        format!("{}/3/device/{}", root, device_token)
    }

    /// Send a notification.
    /// Returns the UUID (either the configured one, or the one returned by the
    /// api).
    pub fn send(&self, notification: Notification) -> Result<Uuid, SendError> {
        let n = notification;

        // Just always generate a uuid client side for simplicity.
        let id = n.id.unwrap_or(Uuid::new_v4());

        if self.delivery_disabled {
            return Ok(id);
        }

        let url = self.build_url(&n.device_token);

        // Add headers.

        let mut headers = List::new();

        // NOTE: if an option which requires a header is not set,
        // the header is still added, but with an empty value,
        // which instructs curl to drop the header.
        // Otherwhise, headers from previous runs would stick around.

        headers.append(&format!("apns-id:{}", id.to_string(),))?;
        headers.append(&format!(
            "apns-expiration:{}",
            n.expiration
                .map(|x| x.to_string())
                .unwrap_or("".to_string())
        ))?;
        headers.append(&format!(
            "apns-priority:{}",
            n.priority
                .map(|x| x.to_int().to_string())
                .unwrap_or("".to_string())
        ))?;
        headers.append(&format!("apns-topic:{}", n.topic))?;
        headers.append(&format!(
            "apns-collapse-id:{}",
            n.collapse_id
                .map(|x| x.as_str().to_string())
                .unwrap_or("".to_string())
        ))?;

        let request = ApnsRequest { aps: n.payload };
        let raw_request = ::serde_json::to_vec(&request)?;

        let mut easy = self.easy.borrow_mut();

        match &self.auth {
            _ => {}
        }

        easy.verbose(self.verbose)?;
        easy.http_headers(headers)?;
        easy.post(true)?;
        easy.post_fields_copy(&raw_request)?;
        easy.url(&url)?;
        easy.perform()?;

        let status = easy.response_code()?;
        if status != 200 {
            // Request failed.
            // Read json response with the error.
            let response_data = easy.get_ref();
            let reason = ErrorResponse::parse_payload(&response_data.0);
            Err(ApiError { status, reason }.into())
        } else {
            Ok(id)
        }
    }
}

#[cfg(test)]
mod test {
    use std::env::var;
    use super::*;

    #[test]
    fn test_cert() {
        let cert_path = var("APNS_CERT_PATH").unwrap();
        let cert_pw = Some(var("APNS_CERT_PW").unwrap());
        let topic = var("APNS_TOPIC").unwrap();
        let token = var("APNS_DEVICE_TOKEN").unwrap();

        let mut apns = ApnsSync::with_certificate(cert_path, cert_pw).unwrap();
        apns.set_verbose(true);
        let n = NotificationBuilder::new(topic, token)
            .title("title")
            .build();
        apns.send(n).unwrap();
    }
}

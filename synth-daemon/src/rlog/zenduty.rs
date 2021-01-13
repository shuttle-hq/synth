use log::{Level, Metadata, Record};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tide::http::Url;

// https://www.zenduty.com/api/events/<integration-key>/
const ENDPOINT: &str = "https://www.zenduty.com/api/events";

pub(crate) struct ZenDuty {
    api_key: String,
    client_blocking: Arc<Client>,
}

// name	        type	default	example	                                                                        required
// message	    string	None	This becomes the incident title	                                                yes
// summary	    string	None	This is the incident summary	                                                no
// alert_type	string	info	Choices - critical, acknowledged, resolved, error, warning, info	            yes
// suppressed	boolean	False	true or false	                                                                no
// entity_id	string	None	A unique id for the alert. If not provided, the Zenduty API will create one	    no
// payload	    json	None	A JSON payload containing additional information about the alert	            no
#[derive(Serialize, Deserialize)]
struct MessageBody {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    alert_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    suppressed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
}

impl ZenDuty {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client_blocking: Arc::new(Client::new()),
        }
    }

    fn build_msg(&self, record: &Record) -> MessageBody {
        let alert_type = match record.level() {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Info => "info",
            Level::Debug => "info",
            Level::Trace => "info",
        };

        MessageBody {
            message: format!("{}", record.args()),
            summary: None,
            alert_type: alert_type.to_string(),
            suppressed: None,
            entity_id: None,
            payload: None,
        }
    }

    fn build_endpoint(&self) -> Url {
        let url_str = format!("{}/{}/", ENDPOINT, self.api_key);
        Url::from_str(&url_str).unwrap() // Unwrap is OK here
    }

    fn log_async(client: Arc<Client>, url: Url, body: MessageBody) {
        std::thread::spawn(move || match client.post(url).json(&body).send() {
            Ok(resp) => match resp.status() {
                StatusCode::CREATED => {
                    trace!("Sent error to Zenduty")
                }
                status => {
                    warn!(
                        "Failed to send message to Zenduty with status code {} ",
                        status
                    );
                }
            },
            Err(e) => {
                error!("Failed to send message to Zenduty. With error {}", e);
            }
        });
    }
}

impl log::Log for ZenDuty {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !(self.enabled(record.metadata())) {
            return;
        }
        let url = self.build_endpoint();
        let body = self.build_msg(record);
        Self::log_async(self.client_blocking.clone(), url, body)
    }

    fn flush(&self) {
        /* Nothing to flush here. We don't buffer any messages */
    }
}

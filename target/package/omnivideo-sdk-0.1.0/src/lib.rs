//! Omni Video Rust SDK — generate video and image content with the **Gemini Omni Video** series of models.
//!
//! Sign in at <https://omnivideo.net/> and create an API key on the account page. Export it as
//! `OMNIVIDEO_API_KEY` and the client picks it up automatically.
//!
//! # Quick start
//!
//! ```no_run
//! use omnivideo_sdk::{OmniVideo, CreateTaskInput};
//!
//! let client = OmniVideo::new(None).unwrap();
//! let task = client.run(CreateTaskInput {
//!     model_id: "seedance-2".into(),
//!     prompt: "a serene zen garden at sunrise".into(),
//!     aspect_ratio: Some("16:9".into()),
//!     ..Default::default()
//! }, None).unwrap();
//! println!("{:?}", task.output_url());
//! ```

use serde::{Deserialize, Serialize};
use std::env;
use std::thread;
use std::time::{Duration, Instant};

/// Default base URL for the Omni Video API.
pub const DEFAULT_BASE_URL: &str = "https://omnivideo.net/api/v1";

/// State of a generation job.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub task_id: String,
    pub task_status: u8,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub video_url: Option<String>,
    #[serde(default)]
    pub credits: Option<i64>,
    #[serde(default)]
    pub msg: Option<String>,
}

impl Task {
    pub const STATUS_QUEUED: u8 = 1;
    pub const STATUS_RUNNING: u8 = 2;
    pub const STATUS_SUCCESS: u8 = 3;
    pub const STATUS_FAILED: u8 = 4;

    /// Whether the task has reached a terminal state (success or failure).
    pub fn is_done(&self) -> bool {
        self.task_status == Self::STATUS_SUCCESS || self.task_status == Self::STATUS_FAILED
    }

    /// Returns `video_url` if present, else `image_url`.
    pub fn output_url(&self) -> Option<&str> {
        self.video_url.as_deref().or(self.image_url.as_deref())
    }
}

/// Request body for `POST /tasks/create`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTaskInput {
    pub model_id: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
}

/// Polling configuration for `OmniVideo::run`.
#[derive(Debug, Clone)]
pub struct RunOptions {
    pub poll_interval: Duration,
    pub max_wait: Duration,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(3),
            max_wait: Duration::from_secs(600),
        }
    }
}

/// SDK error type.
#[derive(Debug)]
pub enum Error {
    MissingApiKey,
    Transport(String),
    Api {
        code: Option<i64>,
        status: Option<u16>,
        message: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingApiKey => write!(
                f,
                "missing API key — pass via builder or set OMNIVIDEO_API_KEY. Get one at https://omnivideo.net/"
            ),
            Error::Transport(m) => write!(f, "transport error: {m}"),
            Error::Api { code, status, message } => {
                write!(f, "api error")?;
                if let Some(s) = status {
                    write!(f, " status={s}")?;
                }
                if let Some(c) = code {
                    write!(f, " code={c}")?;
                }
                write!(f, ": {message}")
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Client for the Omni Video API.
pub struct OmniVideo {
    api_key: String,
    base_url: String,
    agent: ureq::Agent,
}

impl OmniVideo {
    /// Construct a new client. If `api_key` is `None`, reads `OMNIVIDEO_API_KEY` from the environment.
    pub fn new(api_key: Option<&str>) -> Result<Self> {
        let key = api_key
            .map(str::to_owned)
            .or_else(|| env::var("OMNIVIDEO_API_KEY").ok())
            .filter(|s| !s.is_empty())
            .ok_or(Error::MissingApiKey)?;
        Ok(Self {
            api_key: key,
            base_url: DEFAULT_BASE_URL.to_owned(),
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(60))
                .build(),
        })
    }

    /// Override the base URL (useful for testing or self-hosting).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    /// Submit a generation job.
    pub fn create_task(&self, input: &CreateTaskInput) -> Result<Task> {
        let value = self.request("POST", "/tasks/create", Some(serde_json::to_value(input).unwrap()))?;
        parse_task(value)
    }

    /// Fetch the current state of a task.
    pub fn get_task(&self, task_id: &str) -> Result<Task> {
        let value = self.request("GET", &format!("/tasks/{}", task_id), None)?;
        parse_task(value)
    }

    /// Create a task and poll until it reaches a terminal state.
    pub fn run(&self, input: CreateTaskInput, options: Option<RunOptions>) -> Result<Task> {
        let opts = options.unwrap_or_default();
        let mut task = self.create_task(&input)?;
        let deadline = Instant::now() + opts.max_wait;
        while !task.is_done() {
            if Instant::now() > deadline {
                return Err(Error::Api {
                    code: None,
                    status: None,
                    message: format!(
                        "task {} did not finish within {:?}",
                        task.task_id, opts.max_wait
                    ),
                });
            }
            thread::sleep(opts.poll_interval);
            task = self.get_task(&task.task_id)?;
        }
        if task.task_status == Task::STATUS_FAILED {
            return Err(Error::Api {
                code: Some(Task::STATUS_FAILED as i64),
                status: None,
                message: task
                    .msg
                    .clone()
                    .unwrap_or_else(|| format!("task {} failed", task.task_id)),
            });
        }
        Ok(task)
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let req = self
            .agent
            .request(method, &url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Accept", "application/json");
        let response = match body {
            Some(b) => req.send_json(b),
            None => req.call(),
        };
        match response {
            Ok(r) => {
                let payload: serde_json::Value = r
                    .into_json()
                    .map_err(|e| Error::Transport(format!("invalid JSON: {e}")))?;
                if let Some(code) = payload.get("code").and_then(|v| v.as_i64()) {
                    if code != 200 {
                        let message = payload
                            .get("msg")
                            .and_then(|v| v.as_str())
                            .unwrap_or("business error")
                            .to_owned();
                        return Err(Error::Api {
                            code: Some(code),
                            status: None,
                            message,
                        });
                    }
                }
                Ok(payload)
            }
            Err(ureq::Error::Status(status, resp)) => {
                if status == 401 {
                    return Err(Error::Api {
                        code: None,
                        status: Some(401),
                        message: "unauthorized — check your OMNIVIDEO_API_KEY (https://omnivideo.net/)".to_owned(),
                    });
                }
                let body = resp.into_string().unwrap_or_default();
                let msg = serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| v.get("msg").and_then(|m| m.as_str().map(str::to_owned)))
                    .unwrap_or_else(|| format!("HTTP {status}"));
                Err(Error::Api {
                    code: None,
                    status: Some(status),
                    message: msg,
                })
            }
            Err(e) => Err(Error::Transport(e.to_string())),
        }
    }
}

fn parse_task(v: serde_json::Value) -> Result<Task> {
    serde_json::from_value(v).map_err(|e| Error::Transport(format!("invalid task payload: {e}")))
}

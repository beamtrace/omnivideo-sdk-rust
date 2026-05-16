use omnivideo_sdk::{Error, OmniVideo, Task};

#[test]
fn missing_api_key_returns_error() {
    std::env::remove_var("OMNIVIDEO_API_KEY");
    let result = OmniVideo::new(None);
    assert!(matches!(result, Err(Error::MissingApiKey)));
}

#[test]
fn task_helpers_work() {
    let task = Task {
        task_id: "abc".into(),
        task_status: Task::STATUS_SUCCESS,
        image_url: Some("https://x/y.png".into()),
        video_url: None,
        credits: Some(15),
        msg: None,
    };
    assert!(task.is_done());
    assert_eq!(task.output_url(), Some("https://x/y.png"));

    let queued = Task {
        task_id: "abc".into(),
        task_status: Task::STATUS_QUEUED,
        image_url: None,
        video_url: None,
        credits: None,
        msg: None,
    };
    assert!(!queued.is_done());
    assert_eq!(queued.output_url(), None);
}

#[test]
fn video_url_preferred_over_image_url() {
    let task = Task {
        task_id: "abc".into(),
        task_status: Task::STATUS_SUCCESS,
        image_url: Some("https://x/y.png".into()),
        video_url: Some("https://x/y.mp4".into()),
        credits: None,
        msg: None,
    };
    assert_eq!(task.output_url(), Some("https://x/y.mp4"));
}

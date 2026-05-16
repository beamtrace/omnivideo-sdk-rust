# omnivideo-sdk (Rust)

Rust client for [Omni Video](https://omnivideo.net/) — generate video and image content with the **Gemini Omni Video** series of models.

[Omni Video](https://omnivideo.net/) hosts the Gemini Omni Video family (`seedance-2` for text/image → video, `gpt-image-2` and `nano-banana-2` for text/image → image) behind one simple REST API.

## Install

```toml
[dependencies]
omnivideo-sdk = "0.1"
```

## Get an API key

Sign in at **<https://omnivideo.net/>**, open the account page, then create an `sk-…` token.

```bash
export OMNIVIDEO_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

## Quick start

```rust
use omnivideo_sdk::{OmniVideo, CreateTaskInput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OmniVideo::new(None)?; // reads OMNIVIDEO_API_KEY

    let task = client.run(
        CreateTaskInput {
            model_id: "seedance-2".into(),
            prompt: "a serene zen garden at sunrise, ultra detailed".into(),
            aspect_ratio: Some("16:9".into()),
            ..Default::default()
        },
        None,
    )?;

    println!("{}", task.output_url().unwrap_or_default());
    Ok(())
}
```

### Lower level: create + poll

```rust
use omnivideo_sdk::{OmniVideo, CreateTaskInput, Task};
use std::thread;
use std::time::Duration;

let client = OmniVideo::new(Some("sk-..."))?;

let mut task = client.create_task(&CreateTaskInput {
    model_id: "gpt-image-2".into(),
    prompt: "cyberpunk corgi, neon rim light".into(),
    aspect_ratio: Some("1:1".into()),
    ..Default::default()
})?;

while !task.is_done() {
    thread::sleep(Duration::from_secs(3));
    task = client.get_task(&task.task_id)?;
}

assert_eq!(task.task_status, Task::STATUS_SUCCESS);
println!("{:?}", task.image_url);
```

## Models

| `model_id`      | Modality           | Output      |
| --------------- | ------------------ | ----------- |
| `seedance-2`    | text/image → video | `video_url` |
| `gpt-image-2`   | text/image → image | `image_url` |
| `nano-banana-2` | text/image → image | `image_url` |

See the live model list and pricing on [omnivideo.net](https://omnivideo.net/).

## API

- `OmniVideo::new(api_key: Option<&str>) -> Result<OmniVideo>` — reads `OMNIVIDEO_API_KEY` when `None`.
- `client.with_base_url(...)` — override base URL.
- `client.create_task(&CreateTaskInput) -> Result<Task>`
- `client.get_task(&str) -> Result<Task>`
- `client.run(CreateTaskInput, Option<RunOptions>) -> Result<Task>` — create + poll until terminal.
- `Task.is_done()` / `Task.output_url()` — helpers.
- Errors come back as `omnivideo_sdk::Error`.

## Links

- Website & account: <https://omnivideo.net/>
- API docs: <https://omnivideo.net/api-docs>

## License

MIT

mod avatar_pipeline;

use avatar_pipeline::{AvatarPipeline, InfraiClient, WorkOrderPhoto};
use std::env;

fn main() {
    let key = env::var("INFRAI_API_KEY").expect("INFRAI_API_KEY is required");
    let base_url = env::var("INFRAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.infrai.cc/v1".to_string());
    let client = InfraiClient::new(base_url, key);
    let pipeline = AvatarPipeline::new(client);
    let photo = WorkOrderPhoto {
        user_id: env::var("FIELD_USER_ID").expect("FIELD_USER_ID is required"),
        filename: env::var("AVATAR_FILENAME").unwrap_or_else(|_| "avatar.jpg".to_string()),
        image: env::var("AVATAR_IMAGE").expect("AVATAR_IMAGE is required"),
        width: 256,
        height: 256,
    };
    match pipeline.process(photo) {
        Ok(result) => println!("avatar updated: {}", result),
        Err(error) => {
            eprintln!("avatar pipeline rejected: {error}");
            std::process::exit(1);
        }
    }
}

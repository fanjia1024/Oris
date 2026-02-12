use oris_runtime::language_models::llm::LLM;
use oris_runtime::llm::Deepseek;
use oris_runtime::schemas::Message;

#[tokio::main]
async fn main() {
    // Initialize the Deepseek client
    // Requires DEEPSEEK_API_KEY environment variable to be set
    let deepseek = Deepseek::new()
        .with_api_key("your_api_key")
        .with_model("deepseek-chat"); // Can use enum: DeepseekModel::DeepseekChat.to_string()

    // Generate a response
    let response = deepseek
        .generate(&[Message::new_human_message("Introduce the Great Wall")])
        .await
        .unwrap();

    println!("Response: {}", response.generation);
}

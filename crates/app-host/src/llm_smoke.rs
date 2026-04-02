use std::env;

use matrixclaw_agent_core::r#loop::run_prompt;
use matrixclaw_agent_core::{RunRequest, RunResult};
use matrixclaw_provider::openai::OpenAiProvider;
use matrixclaw_tools::ToolRegistry;

const EXPECTED_SENTINEL: &str = "MATRIXCLAW_KIMI_SMOKE_OK";

pub async fn run_openrouter_smoke(model: &str) -> Result<String, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY is not set in the environment".to_string())?;

    let mut provider = OpenAiProvider::for_openrouter(api_key, model).map_err(|error| error.0)?;

    let request = RunRequest::new(format!(
        "Reply with exactly `{EXPECTED_SENTINEL}` and nothing else."
    ));

    let registry = ToolRegistry::new();
    let result = run_prompt(&mut provider, &request, &registry, &mut |_| {})
        .await
        .map_err(|error| error.0)?;
    validate_result(&result)?;

    Ok(format!(
        "model={model}\nstreamed_message={}\nfinal_message={}",
        result.streamed_message.trim(),
        result.final_message.trim(),
    ))
}

fn validate_result(result: &RunResult) -> Result<(), String> {
    let streamed = result.streamed_message.trim();
    let final_message = result.final_message.trim();

    if streamed != EXPECTED_SENTINEL {
        return Err(format!(
            "unexpected streamed message from provider: expected `{EXPECTED_SENTINEL}`, got `{streamed}`"
        ));
    }

    if final_message != EXPECTED_SENTINEL {
        return Err(format!(
            "unexpected final message from provider: expected `{EXPECTED_SENTINEL}`, got `{final_message}`"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_result, EXPECTED_SENTINEL};
    use matrixclaw_agent_core::RunResult;

    #[test]
    fn accepts_exact_sentinel() {
        let result = RunResult {
            streamed_message: EXPECTED_SENTINEL.to_string(),
            final_message: EXPECTED_SENTINEL.to_string(),
            tool_calls_made: 0,
            iterations: 1,
        };

        assert!(validate_result(&result).is_ok());
    }

    #[test]
    fn rejects_extra_text() {
        let result = RunResult {
            streamed_message: format!("{EXPECTED_SENTINEL}."),
            final_message: EXPECTED_SENTINEL.to_string(),
            tool_calls_made: 0,
            iterations: 1,
        };

        assert!(validate_result(&result).is_err());
    }
}

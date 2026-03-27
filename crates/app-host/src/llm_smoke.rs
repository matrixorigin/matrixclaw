use std::env;

use matrixclaw_agent_core::r#loop::run_prompt_with_trace;
use matrixclaw_agent_core::{RunRequest, RunResult};

use crate::openai_compatible::OpenAiCompatibleProvider;

const EXPECTED_SENTINEL: &str = "MATRIXCLAW_KIMI_SMOKE_OK";

pub fn run_openrouter_smoke(model: &str) -> Result<String, String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY is not set in the environment".to_string())?;

    let mut provider =
        OpenAiCompatibleProvider::for_openrouter(api_key, model).map_err(|error| error.0)?;

    let request = RunRequest::new(format!(
        "Reply with exactly `{EXPECTED_SENTINEL}` and nothing else."
    ));

    let trace = run_prompt_with_trace(&mut provider, &request, None).map_err(|error| error.0)?;
    validate_result(&trace.result)?;

    Ok(format!(
        "model={model}\nstreamed_message={}\nfinal_message={}\nevents={:?}",
        trace.result.streamed_message.trim(),
        trace.result.final_message.trim(),
        trace.events
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
        };

        assert!(validate_result(&result).is_ok());
    }

    #[test]
    fn rejects_extra_text() {
        let result = RunResult {
            streamed_message: format!("{EXPECTED_SENTINEL}."),
            final_message: EXPECTED_SENTINEL.to_string(),
        };

        assert!(validate_result(&result).is_err());
    }
}

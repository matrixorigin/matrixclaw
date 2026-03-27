pub const CHALLENGE_TOKEN: &str = "challenge-token";

pub fn validate_response(token: &str) -> bool {
    token == CHALLENGE_TOKEN
}

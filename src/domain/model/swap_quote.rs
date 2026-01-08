use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub from_token: String,
    pub to_token: String,
    pub input_amount: String,
    pub estimated_output: String,
    pub gas_estimate: String,
    pub simulation_success: bool,
    pub error_message: Option<String>,
}

impl SwapQuote {
    pub fn success(
        from_token: String,
        to_token: String,
        input_amount: String,
        estimated_output: String,
        gas_estimate: String,
    ) -> Self {
        Self {
            from_token,
            to_token,
            input_amount,
            estimated_output,
            gas_estimate,
            simulation_success: true,
            error_message: None,
        }
    }

    pub fn failure(
        from_token: String,
        to_token: String,
        input_amount: String,
        error: String,
    ) -> Self {
        Self {
            from_token,
            to_token,
            input_amount,
            estimated_output: "0".to_string(),
            gas_estimate: "0".to_string(),
            simulation_success: false,
            error_message: Some(error),
        }
    }
}

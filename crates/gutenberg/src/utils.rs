use crate::err::{self, GutenError};

pub fn validate_address(input: &String) -> Result<(), GutenError> {
    if input == "sui::tx_context::sender(ctx)" {
        return Ok(());
    }

    let hexa_str = input.strip_prefix("0x").unwrap_or(input);
    let hexa = hex::decode(hexa_str).map_err(|err| {
        err::invalid_address(
            hexa_str.to_string(),
            format!(
                "Failed with the following error: {}
Failed to decode hexadecimal string `{}`",
                err, hexa_str,
            ),
        )
    })?;

    if hexa.len() != 32 {
        Err(err::invalid_address(
            hexa_str.to_string(),
            format!(
                "Invalid Hexadecimal number. Expected 32 digits, got {}",
                hexa.len(),
            ),
        ))
    } else {
        Ok(())
    }
}

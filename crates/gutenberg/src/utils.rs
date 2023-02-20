pub fn address_validator(input: &String) -> Result<(), GutenError> {
    if input == DEFAULT_ADDRESS {
        return Ok(());
    }

    let hexa_str = input.strip_prefix("0x").unwrap_or(input);
    let hexa = hex::decode(hexa_str)?;
    if hexa.len() != 20 {
        err::invalid_address(
            hexa,
            "Invalid Hexadecimal number. Expected 20 digits, got {}",
            hexa.len(),
        )
    } else {
        Ok(())
    }
}

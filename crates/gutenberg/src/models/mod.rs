pub mod collection;
pub mod launchpad;
pub mod nft;

/// Generic method to write move functions
///
/// Use of this function is discouraged unless there is a wide range of
/// function permutations.
pub fn write_move_fn<F>(
    name: &str,
    params: &[&str],
    param_types: &[&str],
    is_public: bool,
    is_entry: bool,
    returns: Option<String>,
    body_fn: F,
) -> String
where
    F: FnOnce() -> String,
{
    let is_public_str = match is_public {
        true => "public ",
        false => "",
    };

    let is_entry_str = match is_entry {
        true => "entry ",
        false => "",
    };

    let args_str = params
        .iter()
        .zip(param_types)
        .map(|(param, param_type)| format!("        {param}: {param_type},\n"))
        .collect::<Vec<String>>()
        .join("");

    let returns_str = returns
        .map(|returns| format!(": {returns}"))
        .unwrap_or_default();
    let body_str = body_fn();

    format!(
        "

    {is_public_str}{is_entry_str}fun {name}(
{args_str}    ){returns_str} {{{body_str}
    }}"
    )
}

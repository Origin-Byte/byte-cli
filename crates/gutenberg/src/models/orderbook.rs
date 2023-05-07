#[derive(Deserialize)]
enum Orderbook {
    None,
    Unprotected,
    Protected,
}

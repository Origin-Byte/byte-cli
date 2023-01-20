// TODO: Harcode the data here!
use rust_sdk::mint;

pub async fn mint_nfts(contract_id: String, _gas_budget: usize) {
    let client = mint::get_client().await.unwrap();
    let keystore = mint::get_keystore().await.unwrap();

    let warehouse_id =
        mint::create_warehouse(&client, &keystore).await.unwrap();

    let _nft_id_1 = mint::mint_nft(
        &client,
        &keystore,
        "suimarines-1",
        "Suimarine #1",
        "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/1.png",
        warehouse_id.as_str(),
        contract_id.as_str(),
    )
    .await
    .unwrap();

    let _nft_id_2 = mint::mint_nft(
        &client,
        &keystore,
        "suimarines-2",
        "Suimarine #2",
        "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/2.png",
        warehouse_id.as_str(),
        contract_id.as_str(),
    )
    .await
    .unwrap();

    let _nft_id_3 = mint::mint_nft(
        &client,
        &keystore,
        "suimarines-3",
        "Suimarine #3",
        "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/3.png",
        warehouse_id.as_str(),
        contract_id.as_str(),
    )
    .await
    .unwrap();

    let _nft_id_4 = mint::mint_nft(
        &client,
        &keystore,
        "suimarines-4",
        "Suimarine #4",
        "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/4.png",
        warehouse_id.as_str(),
        contract_id.as_str(),
    )
    .await
    .unwrap();

    println!("NFT #1 successfully minted.");
    println!("NFT #2 successfully minted.");
    println!("NFT #3 successfully minted.");
    println!("NFT #4 successfully minted.");
}

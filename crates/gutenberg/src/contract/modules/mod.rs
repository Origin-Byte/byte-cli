use crate::Schema;

pub mod protocol;
pub mod standard;
pub mod sui;

pub use protocol::{
    CollectionMod, ComposableNftMod, CreatorsMod, DisplayMod, MintCapMod,
    NftMod, RoyaltiesMod, RoyaltyMod, TagsMod, TemplateMod, TemplatesMod,
    WarehouseMod, WitnessMod,
};
pub use standard::StringMod;
pub use sui::{Balance, Transfer, TxContext, Url, VecMap, VecSet};

pub trait Module {
    fn import(&self) -> String;
}

pub struct Modules {
    string: bool,
    vec_set: bool,
    vec_map: bool,
    url: bool,
    balance: bool,
    transfer: bool,
    tx_context: bool,
    nft: bool,
    witness: bool,
    mint_cap: bool,
    collection: bool,
    tags: bool,
    royalty: bool,
    display: bool,
    creators: bool,
    warehouse: bool,
    template: bool,
    templates: bool,
    composable_nft: bool,
    royalties: bool,
}

pub struct Imports {
    modules: Modules,
    code: String,
}

impl Modules {
    pub fn from_schema(schema: &Schema) -> Self {
        Modules {
            string: true,
            vec_set: !schema.collection.creators.is_empty(),
            vec_map: schema.settings.royalties.is_some(),
            url: schema.collection.url.is_some() || schema.nft.url,
            balance: schema.settings.royalties.is_some(),
            transfer: true,
            tx_context: true,
            nft: true,
            witness: true,
            mint_cap: true,
            collection: true,
            tags: schema.settings.tags.is_some(),
            royalty: schema.settings.royalties.is_some(),
            display: schema.nft.has_display_domains(),
            creators: true,
            warehouse: schema.settings.mint_policies.launchpad
                && !schema.settings.loose,
            template: schema.settings.loose,
            templates: schema.settings.loose,
            composable_nft: schema.settings.composability.is_some(),
            royalties: schema.settings.royalties.is_some(),
        }
    }
}

impl Imports {
    pub fn from_schema(schema: &Schema) -> Self {
        Imports {
            modules: Modules::from_schema(schema),
            code: "".to_string(),
        }
    }

    pub fn write_imports(&mut self) -> String {
        if self.modules.string {
            self.write_import(StringMod::default());
        }
        if self.modules.vec_set {
            self.write_import(VecSet::default());
        }
        if self.modules.vec_map {
            self.write_import(VecMap::default());
        }
        if self.modules.url {
            self.write_import(Url::default());
        }
        if self.modules.balance {
            self.write_import(Balance::default());
        }
        if self.modules.transfer {
            self.write_import(Transfer::default());
        }
        if self.modules.tx_context {
            self.write_import(TxContext::default());
        }
        if self.modules.nft {
            self.write_import(NftMod::default());
        }
        if self.modules.witness {
            self.write_import(WitnessMod::default());
        }
        if self.modules.mint_cap {
            self.write_import(MintCapMod::default());
        }
        if self.modules.collection {
            self.write_import(CollectionMod::default());
        }
        if self.modules.tags {
            self.write_import(TagsMod::default());
        }
        if self.modules.royalty {
            self.write_import(RoyaltyMod::default());
        }
        if self.modules.display {
            self.write_import(DisplayMod::default());
        }
        if self.modules.creators {
            self.write_import(CreatorsMod::default());
        }
        if self.modules.warehouse {
            self.write_import(WarehouseMod::default());
        }
        if self.modules.template {
            self.write_import(TemplateMod::default());
        }
        if self.modules.templates {
            self.write_import(TemplatesMod::default());
        }
        if self.modules.composable_nft {
            self.write_import(ComposableNftMod::default());
        }
        if self.modules.royalties {
            self.write_import(RoyaltiesMod::default());
        }

        self.code.clone()
    }

    pub fn write_import(&mut self, module: impl Module) {
        self.code.push_str(module.import().as_str());
    }
}

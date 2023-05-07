use crate::Schema;

pub mod protocol;
pub mod standard;
pub mod sui;

pub use protocol::{
    CollectionMod, ComposableNftMod, CreatorsMod, DisplayInfoMod, MintCapMod,
    NftMod, RoyaltiesMod, RoyaltyMod, TagsMod, TemplateMod, TemplatesMod,
    WarehouseMod, WitnessMod,
};
pub use standard::StringMod;
pub use sui::{Balance, Transfer, TxContext, Url, VecMap, VecSet};

pub trait Module {
    fn import(&self, has_self: bool) -> String;
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
            url: true,
            balance: schema.settings.royalties.is_some(),
            transfer: true,
            tx_context: true,
            nft: true,
            witness: true,
            mint_cap: true,
            collection: true,
            tags: schema.settings.tags.is_some(),
            royalty: schema.settings.royalties.is_some(),
            display: true,
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

    pub fn write_imports(&mut self, schema: &Schema) -> String {
        if self.modules.string {
            self.write_import(StringMod::default(), true);
        }
        if self.modules.vec_set {
            self.write_import(VecSet::default(), false);
        }
        if self.modules.vec_map {
            self.write_import(VecMap::default(), false);
        }
        if self.modules.url {
            self.write_import(Url::default(), false);
        }
        if self.modules.balance {
            self.write_import(Balance::default(), false);
        }
        if self.modules.transfer {
            self.write_import(Transfer::default(), false);
        }
        if self.modules.tx_context {
            let creator_is_sender = schema.collection.creators.is_empty();
            self.write_import(TxContext::default(), creator_is_sender);
        }
        if self.modules.nft {
            self.write_import(NftMod::default(), true);
        }
        if self.modules.witness {
            self.write_import(WitnessMod::default(), true);
        }
        if self.modules.mint_cap {
            self.write_import(MintCapMod::default(), false);
        }
        if self.modules.collection {
            self.write_import(CollectionMod::default(), true);
        }
        if self.modules.tags {
            self.write_import(TagsMod::default(), true);
        }
        if self.modules.royalty {
            self.write_import(RoyaltyMod::default(), true);
        }
        if self.modules.display {
            self.write_import(DisplayInfoMod::default(), true);
        }
        if self.modules.creators {
            self.write_import(CreatorsMod::default(), true);
        }
        if self.modules.warehouse {
            self.write_import(WarehouseMod::default(), true);
        }
        if self.modules.template {
            self.write_import(TemplateMod::default(), true);
        }
        if self.modules.templates {
            self.write_import(TemplatesMod::default(), true);
        }
        if self.modules.composable_nft {
            self.write_import(ComposableNftMod::default(), true);
        }
        if self.modules.royalties {
            self.write_import(RoyaltiesMod::default(), true);
        }

        self.code.clone()
    }

    pub fn write_import(&mut self, module: impl Module, has_self: bool) {
        self.code.push_str(module.import(has_self).as_str());
    }
}

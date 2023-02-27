use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::contract::modules::ComposableNftMod;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Composability {
    types: BTreeSet<String>,
    blueprint: HashMap<String, Child>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Child {
    pub child_type: String,
    pub order: u64,
    pub limit: u64,
}

impl Child {
    pub fn new(child_type: String, order: u64, limit: u64) -> Self {
        Child {
            child_type,
            order,
            limit,
        }
    }
}

impl Composability {
    pub fn new_from_tradeable_traits(
        types: BTreeSet<String>,
        core_trait: String,
    ) -> Self {
        let mut traits_ = types.clone();
        traits_.retain(|trait_| trait_ != &core_trait);

        let mut blueprint = HashMap::new();
        let mut i = 1;
        for trait_ in traits_.iter() {
            blueprint
                .insert(core_trait.clone(), Child::new(trait_.clone(), i, 1));

            i += 1;
        }

        Composability { types, blueprint }
    }

    pub fn write_types(&self) -> String {
        let mut types = String::new();

        self.types
            .iter()
            .map(|t| {
                types.push_str(ComposableNftMod::add_type(t).as_str());
            })
            .for_each(drop);

        types
    }

    pub fn write_domain(&self) -> String {
        let mut code = ComposableNftMod::init_blueprint();

        self.blueprint
            .iter()
            .map(|(parent_type, child)| {
                code.push_str(
                    format!(
                        "
        c_nft::add_relationship<{parent_type}, {child_type}>(
            &mut blueprint,
            {limit}, // limit
            {order}, // order
            ctx
        );\n",
                        parent_type = parent_type,
                        child_type = child.child_type,
                        limit = child.limit,
                        order = child.order,
                    )
                    .as_str(),
                );
            })
            .for_each(drop);

        code.push_str(ComposableNftMod::add_collection_domain().as_str());

        code
    }
}

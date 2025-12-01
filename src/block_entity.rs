use fastnbt::{IntArray, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

macro_rules! hashmap {
    () => {
        std::collections::HashMap::new()
    };
    ($($k:expr => $v:expr),+ $(,)?) => {
        {
            let mut hashmap = std::collections::HashMap::new();
            $(
                hashmap.insert($k, $v);
            )+
            hashmap
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEntityData {
    pub id: String,
    #[serde(rename = "camelCase")]
    pub keep_packed: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub components: HashMap<String, Value>,
    #[serde(flatten)]
    pub data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockEntity {
    Banner,
    Barrel,
    Beacon,
    Bed,
    Beehive,
    Bell,
    BlastFurnace,
    BrewingStand,
    BrushableBlock,
    CalibratedSculkSensor,
    Campfire,
    Chest,
    ChiseledBookshelf,
    Comparator,
    CommandBlock,
    Conduit,
    CopperGolemStatue,
    Crafter,
    CreakingHeart,
    DaylightDetector,
    DecoratedPot,
    Dispenser,
    Dropper,
    EnchantingTable,
    EnderChest,
    EndGateway,
    EndPortal,
    Furnace,
    HangingSign,
    Hopper,
    Jigsaw,
    Jukebox,
    Lectern,
    MobSpawner,
    Piston,
    ShulkerBox,
    Sign,
    Skull,
    SculkCatalyst,
    SculkSensor,
    SculkShrieker,
    Smoker,
    StructureBlock,
    TrappedChest,
    TrialSpawner,
    Vault,
}

impl ToString for BlockEntity {
    fn to_string(&self) -> String {
        String::from(match self {
            BlockEntity::Banner => "banner",
            BlockEntity::Barrel => "barrel",
            BlockEntity::Beacon => "beacon",
            BlockEntity::Bed => "bed",
            BlockEntity::Beehive => "beehive",
            BlockEntity::Bell => "bell",
            BlockEntity::BlastFurnace => "blast_furnace",
            BlockEntity::BrewingStand => "brewing_stand",
            BlockEntity::BrushableBlock => "brushable_block",
            BlockEntity::CalibratedSculkSensor => "calibrated_sculk_sensor",
            BlockEntity::Campfire => "campfire",
            BlockEntity::Chest => "chest",
            BlockEntity::ChiseledBookshelf => "chiseled_bookshelf",
            BlockEntity::Comparator => "comparator",
            BlockEntity::CommandBlock => "command_block",
            BlockEntity::Conduit => "conduit",
            BlockEntity::CopperGolemStatue => "copper_golem_statue",
            BlockEntity::Crafter => "crafter",
            BlockEntity::CreakingHeart => "creaking_heart",
            BlockEntity::DaylightDetector => "daylight_detector",
            BlockEntity::DecoratedPot => "decorated_pot",
            BlockEntity::Dispenser => "dispenser",
            BlockEntity::Dropper => "dropper",
            BlockEntity::EnchantingTable => "enchanting_table",
            BlockEntity::EnderChest => "ender_chest",
            BlockEntity::EndGateway => "end_gateway",
            BlockEntity::EndPortal => "end_portal",
            BlockEntity::Furnace => "furnace",
            BlockEntity::HangingSign => "hanging_sign",
            BlockEntity::Hopper => "hopper",
            BlockEntity::Jigsaw => "jigsaw",
            BlockEntity::Jukebox => "jukebox",
            BlockEntity::Lectern => "lectern",
            BlockEntity::MobSpawner => "mob_spawner",
            BlockEntity::Piston => "piston",
            BlockEntity::ShulkerBox => "shulker_box",
            BlockEntity::Sign => "sign",
            BlockEntity::Skull => "skull",
            BlockEntity::SculkCatalyst => "sculk_catalyst",
            BlockEntity::SculkSensor => "sculk_sensor",
            BlockEntity::SculkShrieker => "sculk_shrieker",
            BlockEntity::Smoker => "smoker",
            BlockEntity::StructureBlock => "structure_block",
            BlockEntity::TrappedChest => "trapped_chest",
            BlockEntity::TrialSpawner => "trial_spawner",
            BlockEntity::Vault => "vault",
        })
    }
}

macro_rules! mc_colors {
    ($name:literal) => {
        concat!("minecraft:white_", $name)
            | concat!("minecraft:orange_", $name)
            | concat!("minecraft:magenta_", $name)
            | concat!("minecraft:light_blue_", $name)
            | concat!("minecraft:yellow_", $name)
            | concat!("minecraft:lime_", $name)
            | concat!("minecraft:pink_", $name)
            | concat!("minecraft:gray_", $name)
            | concat!("minecraft:light_gray_", $name)
            | concat!("minecraft:cyan_", $name)
            | concat!("minecraft:purple_", $name)
            | concat!("minecraft:blue_", $name)
            | concat!("minecraft:brown_", $name)
            | concat!("minecraft:green_", $name)
            | concat!("minecraft:red_", $name)
            | concat!("minecraft:black_", $name)
    };
}

macro_rules! mc_planks {
    ($name:literal) => {
        concat!("minecraft:oak_", $name)
            | concat!("minecraft:spruce_", $name)
            | concat!("minecraft:birch_", $name)
            | concat!("minecraft:jungle_", $name)
            | concat!("minecraft:acacia_", $name)
            | concat!("minecraft:dark_oak_", $name)
            | concat!("minecraft:mangrove_", $name)
            | concat!("minecraft:cherry_", $name)
            | concat!("minecraft:pale_oak_", $name)
            | concat!("minecraft:bamboo_", $name)
            | concat!("minecraft:crimson_", $name)
            | concat!("minecraft:warped_", $name)
    };
}

macro_rules! mc {
    ($str:literal) => {
        concat!("minecraft:", $str)
    };
}

pub fn lookup_block_entity_name(block_name: &str) -> Option<BlockEntity> {
    match block_name {
        mc_colors!("banner") | mc_colors!("wall_banner") => Some(BlockEntity::Banner),
        mc!("barrel") => Some(BlockEntity::Barrel),
        mc!("beacon") => Some(BlockEntity::Beacon),
        mc_colors!("bed") => Some(BlockEntity::Bed),
        mc!("beehive") => Some(BlockEntity::Beehive),
        mc!("bell") => Some(BlockEntity::Bell),
        mc!("blast_furnace") => Some(BlockEntity::BlastFurnace),
        mc!("brewing_stand") => Some(BlockEntity::BrewingStand),
        mc!("suspicious_sand") | mc!("suspicious_gravel") => Some(BlockEntity::BrushableBlock),
        mc!("calibrated_sculk_sensor") => Some(BlockEntity::CalibratedSculkSensor),
        mc!("campfire") | mc!("soul_campfire") => Some(BlockEntity::Campfire),
        mc!("chiseled_bookshelf") => Some(BlockEntity::ChiseledBookshelf),
        mc!("chest") => Some(BlockEntity::Chest),
        mc!("comparator") => Some(BlockEntity::Comparator),
        mc!("command_block") | mc!("chain_command_block") | mc!("repeating_command_block") => {
            Some(BlockEntity::CommandBlock)
        }
        mc!("conduit") => Some(BlockEntity::Conduit),
        mc!("copper_golem_statue")
        | mc!("exposed_copper_golem_statue")
        | mc!("weathered_copper_golem_statue")
        | mc!("oxidized_copper_golem_statue")
        | mc!("waxed_copper_golem_statue")
        | mc!("waxed_exposed_copper_golem_statue")
        | mc!("waxed_weathered_copper_golem_statue")
        | mc!("waxed_oxidized_copper_golem_statue") => Some(BlockEntity::CopperGolemStatue),
        mc!("crafter") => Some(BlockEntity::Crafter),
        mc!("creaking_heart") => Some(BlockEntity::CreakingHeart),
        mc!("daylight_detector") => Some(BlockEntity::DaylightDetector),
        mc!("decorated_pot") => Some(BlockEntity::DecoratedPot),
        mc!("dispenser") => Some(BlockEntity::Dispenser),
        mc!("dropper") => Some(BlockEntity::Dropper),
        mc!("enchanting_table") => Some(BlockEntity::EnchantingTable),
        mc!("ender_chest") => Some(BlockEntity::EnderChest),
        mc!("end_gateway") => Some(BlockEntity::EndGateway),
        mc!("end_portal") => Some(BlockEntity::EndPortal),
        mc!("furnace") => Some(BlockEntity::Furnace),
        mc_planks!("hanging_sign") | mc_planks!("wall_hanging_sign") => {
            Some(BlockEntity::HangingSign)
        }
        mc!("hopper") => Some(BlockEntity::Hopper),
        mc!("jigsaw") => Some(BlockEntity::Jigsaw),
        mc!("jukebox") => Some(BlockEntity::Jukebox),
        mc!("lectern") => Some(BlockEntity::Lectern),
        mc!("spawner") => Some(BlockEntity::MobSpawner),
        mc!("moving_piston") => Some(BlockEntity::Piston),
        mc!("sculk_catalyst") => Some(BlockEntity::SculkCatalyst),
        mc!("sculk_sensor") => Some(BlockEntity::SculkSensor),
        mc!("sculk_shrieker") => Some(BlockEntity::SculkShrieker),
        mc!("shulker_box") | mc_colors!("shulker_box") => Some(BlockEntity::ShulkerBox),
        mc_planks!("sign") | mc_planks!("wall_sign") => Some(BlockEntity::Sign),
        mc!("skeleton_skull")
        | mc!("wither_skeleton_skull")
        | mc!("zombie_head")
        | mc!("player_head")
        | mc!("creeper_head")
        | mc!("dragon_head")
        | mc!("piglin_head")
        | mc!("skeleton_wall_skull")
        | mc!("wither_skeleton_wall_skull")
        | mc!("zombie_wall_head")
        | mc!("player_wall_head")
        | mc!("creeper_wall_head")
        | mc!("dragon_wall_head")
        | mc!("piglin_wall_head") => Some(BlockEntity::Skull),
        mc!("smoker") => Some(BlockEntity::Smoker),
        mc!("structure_block") => Some(BlockEntity::StructureBlock),
        mc!("trapped_chest") => Some(BlockEntity::TrappedChest),
        mc!("trial_spawner") => Some(BlockEntity::TrialSpawner),
        mc!("vault") => Some(BlockEntity::Vault),
        _ => None,
    }
}

pub fn create_default_block_entity(
    block_entity: BlockEntity,
    x: i32,
    y: i32,
    z: i32,
) -> BlockEntityData {
    let mut data = HashMap::<&'static str, Value>::new();
    match block_entity {
        BlockEntity::Barrel => {
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::Beacon => {
            data.insert("Levels", Value::Int(0));
        }
        BlockEntity::Beehive => {
            data.insert("bees", Value::List(vec![]));
        }
        BlockEntity::BlastFurnace => {
            data.insert("lit_time_remaining", Value::Short(0));
            data.insert("cooking_time_spent", Value::Short(0));
            data.insert("cooking_total_time", Value::Short(0));
            data.insert("lit_total_time", Value::Short(0));
            data.insert("Items", Value::List(vec![]));
            data.insert("RecipesUsed", Value::Compound(hashmap!()));
        }
        BlockEntity::BrewingStand => {
            data.insert("BrewTime", Value::Short(0));
            data.insert("Fuel", Value::Byte(0));
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::CalibratedSculkSensor => {
            data.insert("last_vibration_frequency", Value::Int(12));
            data.insert(
                "listener",
                Value::Compound(hashmap!(
                    "selector".to_string() => Value::Compound(hashmap!(
                        "tick".to_string() => Value::Long(-1)
                    )),
                    "event_delay".to_string() => Value::Int(0)
                )),
            );
        }
        BlockEntity::Campfire => {
            data.insert(
                "CookingTimes",
                Value::IntArray(IntArray::new(vec![0, 0, 0, 0])),
            );
            data.insert(
                "CookingTotalTimes",
                Value::IntArray(IntArray::new(vec![0, 0, 0, 0])),
            );
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::ChiseledBookshelf => {
            data.insert("Items", Value::List(vec![]));
            data.insert("last_interacted_slot", Value::Int(-1));
        }
        BlockEntity::Chest => {
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::Comparator => {
            data.insert("OutputSignal", Value::Int(0));
        }
        BlockEntity::CommandBlock => {
            data.insert("auto", Value::Byte(0));
            data.insert("Command", Value::String("".to_string()));
            data.insert("conditionMet", Value::Byte(0));
            data.insert("powered", Value::Byte(0));
            data.insert("SuccessCount", Value::Int(0));
            data.insert("TrackOutput", Value::Byte(1));
            data.insert("UpdateLastExecution", Value::Byte(1));
        }
        BlockEntity::Crafter => {
            data.insert("crafting_ticks_remaining", Value::Int(0));
            data.insert("triggered", Value::Int(0));
            data.insert("disabled_slots", Value::IntArray(IntArray::new(vec![])));
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::Dispenser => {
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::Dropper => {
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::EndGateway => {
            data.insert("Age", Value::Long(0));
        }
        BlockEntity::Furnace => {
            data.insert("lit_time_remaining", Value::Short(0));
            data.insert("cooking_time_spent", Value::Short(0));
            data.insert("cooking_total_time", Value::Short(0));
            data.insert("lit_total_time", Value::Short(0));
            data.insert("Items", Value::List(vec![]));
            data.insert("RecipesUsed", Value::Compound(hashmap!()));
        }
        BlockEntity::HangingSign => {
            data.insert("is_waxed", Value::Byte(0));
            data.insert(
                "front_text",
                Value::Compound(hashmap!(
                    "has_glowing_text".to_string() => Value::Byte(0),
                    "color".to_string() => Value::String("black".to_string()),
                    "messages".to_string() => Value::List(vec![
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string())
                    ])
                )),
            );
            data.insert(
                "back_text",
                Value::Compound(hashmap!(
                    "has_glowing_text".to_string() => Value::Byte(0),
                    "color".to_string() => Value::String("black".to_string()),
                    "messages".to_string() => Value::List(vec![
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string())
                    ])
                )),
            );
        }
        BlockEntity::Hopper => {
            data.insert("Items", Value::List(vec![]));
            data.insert("TransferCooldown", Value::Int(0));
        }
        BlockEntity::Jigsaw => {
            data.insert("final_state", Value::String(mc!("air").to_string()));
            data.insert("joint", Value::String("rollable".to_string()));
            data.insert("name", Value::String(mc!("empty").to_string()));
            data.insert("pool", Value::String(mc!("empty").to_string()));
            data.insert("target", Value::String(mc!("empty").to_string()));
            data.insert("selection_priority", Value::Int(0));
            data.insert("placement_priority", Value::Int(0));
        }
        BlockEntity::MobSpawner => {
            data.insert("Delay", Value::Short(0));
            data.insert("MaxNearbyEntities", Value::Short(6));
            data.insert("MaxSpawnDelay", Value::Short(800));
            data.insert("MinSpawnDelay", Value::Short(200));
            data.insert("RequiredPlayerRange", Value::Short(16));
            data.insert("SpawnCount", Value::Short(4));
            data.insert(
                "SpawnData",
                Value::Compound(hashmap!("entity".to_string() => Value::Compound(hashmap!()))),
            );
            data.insert("SpawnPotentials", Value::List(vec![]));
            data.insert("SpawnRange", Value::Short(4));
        }
        BlockEntity::SculkCatalyst => {
            data.insert("cursors", Value::List(vec![]));
        }
        BlockEntity::SculkSensor => {
            data.insert("last_vibration_frequency", Value::Int(12));
            data.insert(
                "listener",
                Value::Compound(hashmap!(
                    "selector".to_string() => Value::Compound(hashmap!(
                        "tick".to_string() => Value::Long(-1)
                    )),
                    "event_delay".to_string() => Value::Int(0)
                )),
            );
        }
        BlockEntity::SculkShrieker => {
            data.insert("warning_level", Value::Int(0));
            data.insert(
                "listener",
                Value::Compound(hashmap!(
                    "selector".to_string() => Value::Compound(hashmap!(
                        "tick".to_string() => Value::Long(-1)
                    )),
                    "event_delay".to_string() => Value::Int(0)
                )),
            );
        }
        BlockEntity::Sign => {
            data.insert("is_waxed", Value::Byte(0));
            data.insert(
                "front_text",
                Value::Compound(hashmap!(
                    "has_glowing_text".to_string() => Value::Byte(0),
                    "color".to_string() => Value::String("black".to_string()),
                    "messages".to_string() => Value::List(vec![
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string())
                    ])
                )),
            );
            data.insert(
                "back_text",
                Value::Compound(hashmap!(
                    "has_glowing_text".to_string() => Value::Byte(0),
                    "color".to_string() => Value::String("black".to_string()),
                    "messages".to_string() => Value::List(vec![
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string()),
                        Value::String("".to_string())
                    ])
                )),
            );
        }
        BlockEntity::Smoker => {
            data.insert("lit_time_remaining", Value::Short(0));
            data.insert("cooking_time_spent", Value::Short(0));
            data.insert("cooking_total_time", Value::Short(0));
            data.insert("lit_total_time", Value::Short(0));
            data.insert("Items", Value::List(vec![]));
            data.insert("RecipesUsed", Value::Compound(hashmap!()));
        }
        BlockEntity::StructureBlock => {
            data.insert("author", Value::String("notch".to_string()));
            data.insert("ignoreEntities", Value::Byte(1));
            data.insert("integrity", Value::Float(1.0));
            data.insert("metadata", Value::String("".to_string()));
            data.insert("mirror", Value::String("NONE".to_string()));
            data.insert("mode", Value::String("LOAD".to_string()));
            data.insert("name", Value::String("".to_string()));
            data.insert("posX", Value::Int(0));
            data.insert("posY", Value::Int(1));
            data.insert("posZ", Value::Int(0));
            data.insert("powered", Value::Byte(0));
            data.insert("rotation", Value::String("NONE".to_string()));
            data.insert("seed", Value::Long(0));
            data.insert("showboundingbox", Value::Byte(1));
            data.insert("sizeX", Value::Int(0));
            data.insert("sizeY", Value::Int(0));
            data.insert("sizeZ", Value::Int(0));
            data.insert("showair", Value::Byte(0));
            data.insert("strict", Value::Byte(0));
        }
        BlockEntity::TrappedChest => {
            data.insert("Items", Value::List(vec![]));
        }
        BlockEntity::TrialSpawner => {
            data.insert(
                "spawn_data",
                Value::Compound(hashmap!(
                    "entity".to_string() => Value::Compound(hashmap!())
                )),
            );
        }
        BlockEntity::Vault => {
            data.insert(
                "server_data",
                Value::Compound(hashmap!(
                    "state_updating_resumes_at".to_string() => Value::Long(0)
                )),
            );
            data.insert(
                "config",
                Value::Compound(hashmap!(
                    "key_item".to_string() => Value::Compound(hashmap!(
                        "count".to_string() => Value::Int(1),
                        "id".to_string() => Value::String(mc!("trial_key").to_string())
                    ))
                )),
            );
            data.insert("shared_data", Value::Compound(hashmap!()));
        }
        _ => {}
    }
    BlockEntityData {
        id: format!("minecraft:{}", block_entity.to_string()),
        keep_packed: false,
        x,
        y,
        z,
        components: hashmap!(),
        data: data.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
    }
}

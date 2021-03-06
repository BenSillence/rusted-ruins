use super::item::{EquipItemList, ItemList};
use super::map::MapId;
use super::site::SiteId;
use super::skill::SkillList;
use super::unknown_id_err;
use crate::objholder::CharaTemplateIdx;
use std::collections::HashMap;

/// Character's races
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    Animal,
    Human,
    Bug,
    Slime,
    Devil,
    Phantom,
    Ghost,
}

/// Character classes
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[repr(u16)]
#[serde(rename_all = "snake_case")]
pub enum CharaClass {
    None = 0,
    // Playable classes
    Adventurer = 100,
    Rogue,
    Sorcerer,
    Warrior,
    // Npc classes
    Civilian = 200,
}

impl Default for CharaClass {
    fn default() -> CharaClass {
        CharaClass::Civilian
    }
}

/// Relationship between one chara to another.
///         |A|F|N|H
/// ALLY    |A|F|N|H
/// FRIENDLY|F|F|N|H
/// NEUTRAL |N|N|N|N
/// HOSTILE |H|H|N|F
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Relationship {
    ALLY = 0,
    FRIENDLY,
    NEUTRAL,
    HOSTILE,
}

impl Relationship {
    pub fn relative(&self, other: Relationship) -> Relationship {
        use self::Relationship::*;
        match (*self, other) {
            (ALLY, o) => o,
            (FRIENDLY, ALLY) => FRIENDLY,
            (FRIENDLY, FRIENDLY) => FRIENDLY,
            (FRIENDLY, NEUTRAL) => NEUTRAL,
            (FRIENDLY, HOSTILE) => HOSTILE,
            (NEUTRAL, _) => NEUTRAL,
            (HOSTILE, ALLY) => HOSTILE,
            (HOSTILE, FRIENDLY) => HOSTILE,
            (HOSTILE, NEUTRAL) => NEUTRAL,
            (HOSTILE, HOSTILE) => FRIENDLY,
        }
    }
}

/// All data for one character
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chara {
    pub name: Option<String>,
    pub attr: CharaAttributes,
    pub template: CharaTemplateIdx,
    pub class: CharaClass,
    pub level: u32,
    pub item_list: ItemList,
    pub equip: EquipItemList,
    pub wait_time: u32,
    pub ai: CharaAI,
    pub hp: i32,
    pub sp: i32,
    pub status: Vec<CharaStatus>,
    pub skills: SkillList,
    /// Relationship to player character
    pub rel: Relationship,
    /// When talked, execute this script
    pub trigger_talk: Option<String>,
}

/// Character attributes
/// These values are calculated from base params and other factors
/// They are updated by some actions
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CharaAttributes {
    /// Max HP
    pub max_hp: i32,
    /// Strength
    pub str: u16,
    /// Vitality
    pub vit: u16,
    /// Dexterity
    pub dex: u16,
    /// Intelligence
    pub int: u16,
    /// Will
    pub wil: u16,
    /// Charisma
    pub cha: u16,
    /// Speed
    pub spd: u16,
    /// Range of view in tile
    pub view_range: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CharaBaseAttr {
    pub base_hp: i32,
    pub str: i16,
    pub vit: i16,
    pub dex: i16,
    pub int: i16,
    pub wil: i16,
    pub cha: i16,
    pub spd: i16,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CharaAttrRevision {
    pub hp: i32,
    pub str: i16,
    pub vit: i16,
    pub dex: i16,
    pub int: i16,
    pub wil: i16,
    pub cha: i16,
    pub spd: i16,
}

impl CharaBaseAttr {
    pub fn revise(self, r: CharaAttrRevision) -> CharaBaseAttr {
        CharaBaseAttr {
            base_hp: self.base_hp + r.hp,
            str: self.str + r.str,
            vit: self.vit + r.vit,
            dex: self.dex + r.dex,
            int: self.int + r.int,
            wil: self.wil + r.wil,
            cha: self.cha + r.cha,
            spd: self.spd + r.spd,
        }
    }
}

/// Represents chara status
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub enum CharaStatus {
    /// Sp status
    Hungry,
    /// Sp status
    Weak,
    /// Sp status
    Starving,
    Asleep {
        turn_left: u16,
    },
    Poisoned,
}

impl Default for Chara {
    fn default() -> Chara {
        Chara {
            name: None,
            attr: CharaAttributes::default(),
            template: CharaTemplateIdx::default(),
            class: CharaClass::default(),
            level: 0,
            item_list: ItemList::new(),
            equip: EquipItemList::new(&[]),
            wait_time: crate::basic::WAIT_TIME_NUMERATOR,
            ai: CharaAI::default(),
            hp: 100,
            sp: 0,
            status: Vec::new(),
            skills: SkillList::default(),
            rel: Relationship::NEUTRAL,
            trigger_talk: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CharaKind {
    /// Player is unique character in the game
    Player,
    /// Indexed for a site. This character is associated one site.
    /// Citizens on a town use this id.
    OnSite,
    /// Indexed for a map. This character don't appear on other maps.
    /// Randomly generated characters use this id.
    OnMap,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum CharaId {
    /// Player is unique character in the game
    Player,
    /// Indexed for a site. This character is associated one site.
    /// Citizens on a town use this id.
    OnSite { sid: SiteId, n: u32 },
    /// Indexed for a map. This character don't appear on other maps.
    /// Randomly generated characters use this id.
    OnMap { mid: MapId, n: u32 },
}

/// Data to determine NPC character's actions
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct CharaAI {
    pub kind: NpcAIKind,
}

/// Rough kind of NPC AI
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NpcAIKind {
    /// This npc does not do anything.
    None,
    /// This npc will not move
    NoMove,
    /// This npc will chase near enemies, and try melee atacks
    Melee,
}

impl Default for CharaAI {
    fn default() -> CharaAI {
        CharaAI {
            kind: NpcAIKind::None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CharaHolder {
    c: HashMap<CharaId, Chara>,
    on_map: HashMap<CharaId, Chara>,
}

impl CharaHolder {
    pub(crate) fn new() -> CharaHolder {
        CharaHolder {
            c: HashMap::new(),
            on_map: HashMap::new(),
        }
    }

    pub(crate) fn add(&mut self, cid: CharaId, chara: Chara) {
        match cid {
            CharaId::OnMap { .. } => &mut self.on_map,
            _ => &mut self.c,
        }
        .insert(cid, chara);
    }

    pub fn get(&self, cid: CharaId) -> &Chara {
        match cid {
            CharaId::OnMap { .. } => &self.on_map,
            _ => &self.c,
        }
        .get(&cid)
        .unwrap_or_else(|| unknown_id_err(cid))
    }

    pub fn get_mut(&mut self, cid: CharaId) -> &mut Chara {
        match cid {
            CharaId::OnMap { .. } => &mut self.on_map,
            _ => &mut self.c,
        }
        .get_mut(&cid)
        .unwrap_or_else(|| unknown_id_err(cid))
    }

    pub(crate) fn remove_chara(&mut self, cid: CharaId) {
        match cid {
            CharaId::OnMap { .. } => &mut self.on_map,
            _ => &mut self.c,
        }
        .remove(&cid);
    }

    pub(crate) fn replace_on_map_chara(
        &mut self,
        next: HashMap<CharaId, Chara>,
    ) -> HashMap<CharaId, Chara> {
        std::mem::replace(&mut self.on_map, next)
    }
}

/// When a chara is talked to, talk will be start from the section of specified TalkScript
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CharaTalk {
    /// Id of TalkScriptObject
    pub id: String,
    /// Section of the TalkScript
    pub section: String,
}

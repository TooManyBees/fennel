use crate::util::HasKeywords;
use intrusive_collections::{intrusive_adapter, LinkedListLink};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct ObjectId(usize);

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ObjectDef {
    pub id: ObjectId,
    keywords: Vec<String>,
    name: String,
    room_description: String,
    description: Option<String>,
    object_type: ObjectType,
}

intrusive_adapter!(pub ObjectInRoomAdapter = Rc<Object>: Object { in_room_link: LinkedListLink });
intrusive_adapter!(pub ObjectOnCharAdapter = Rc<Object>: Object { on_char_link: LinkedListLink });
intrusive_adapter!(pub AllObjectsAdapter = Rc<Object>: Object { all_objs_link: LinkedListLink });

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Object {
    id: ObjectId,
    keywords: Vec<String>,
    name: String,
    room_description: String,
    description: Option<String>,
    object_type: ObjectType,
    #[serde(skip)]
    in_room_link: LinkedListLink,
    #[serde(skip)]
    on_char_link: LinkedListLink,
    #[serde(skip)]
    all_objs_link: LinkedListLink,
}

impl Object {
    pub fn from_prototype(def: &ObjectDef) -> Object {
        Object {
            id: def.id,
            keywords: def.keywords.clone(),
            name: def.name.clone(),
            room_description: def.room_description.clone(),
            description: def.description.clone(),
            object_type: def.object_type,
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn room_description(&self) -> &str {
        &self.room_description
    }

    pub fn description(&self) -> &str {
        self.description.as_ref().unwrap_or(&self.room_description)
    }
}

impl HasKeywords for Object {
    fn keywords(&self) -> &[String] {
        &self.keywords
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub enum ObjectType {
    Trash,
    Light,
    Clothing,
    Jewelry,
    Armor,
    Weapon,
    Treasure,
    Food,
    Drink,
    Medicine,
    Plant,
    Environment,
    Portal,
    Book,
    Art,
}

impl Default for ObjectType {
    fn default() -> ObjectType {
        ObjectType::Trash
    }
}

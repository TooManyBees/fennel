use crate::area::Area;
use crate::character::{CharId, Character};
use crate::commands::{lookup_command, CommandFn, COMMANDS};
use crate::connection::Connection;
use crate::object::{AllObjectsAdapter, Object, ObjectDef, ObjectId, ObjectInRoomAdapter};
use crate::room::{Room, RoomId};
use crate::util::take_command;
use fnv::FnvHashMap;
use generational_arena::{Arena, Index};
use intrusive_collections::LinkedList;
use std::default::Default;
use std::io::{ErrorKind, Write};
use std::rc::Rc;

pub struct PendingCommand {
    conn_idx: Index,
    at_room: RoomId,
    command: Option<&'static CommandFn>,
    arguments: String,
}

#[derive(Default)]
pub struct World {
    pub connections: Arena<Connection>,
    mark_for_disconnect: Vec<Index>,
    pub areas: Vec<Area>,
    pub npc_defs: FnvHashMap<CharId, Character>,
    pub characters: Arena<Character>,
    pub object_defs: FnvHashMap<ObjectId, ObjectDef>,
    pub objects: LinkedList<AllObjectsAdapter>,
    pub rooms: FnvHashMap<RoomId, Room>,
    pub room_chars: FnvHashMap<RoomId, Vec<Index>>, // Linked list?
    pub room_objs: FnvHashMap<RoomId, LinkedList<ObjectInRoomAdapter>>,
    pending_commands: std::collections::LinkedList<PendingCommand>,
}

impl World {
    pub fn new() -> World {
        let (areas, npc_defs, object_defs, rooms) = load_areas();

        let mut room_chars: FnvHashMap<RoomId, Vec<Index>> = FnvHashMap::default();
        let mut room_objs: FnvHashMap<RoomId, LinkedList<ObjectInRoomAdapter>> =
            FnvHashMap::default();
        for key in rooms.keys() {
            room_chars.insert(*key, vec![]);
            room_objs.insert(*key, Default::default());
        }

        // println!("{:?}\n\n{:?}\n\n{:?}\n\n{:?}", areas, npc_defs, object_defs, rooms);

        World {
            areas,
            npc_defs,
            object_defs,
            rooms,
            room_chars,
            room_objs,
            ..Default::default()
        }
    }

    pub fn populate(&mut self) {
        for npc in self.npc_defs.values() {
            let idx = self.characters.insert(npc.clone());
            self.characters.get_mut(idx).map(|char| char.set_index(idx));
            let in_room = self
                .room_chars
                .get_mut(&npc.in_room)
                .expect("Unwrapped None room chars");
            in_room.push(idx);
        }

        // FIXME: hackity hack-hack
        for room in self.rooms.values() {
            for id in &room.object_ids {
                let obj = Rc::new(Object::from_prototype(&self.object_defs[id]));
                self.objects.push_front(obj.clone());
                let in_room = self
                    .room_objs
                    .get_mut(&room.id)
                    .expect("Unwrapped None room objs");
                in_room.push_front(obj);
            }
        }
    }

    pub fn read_input(&mut self) {
        for (idx, conn) in &mut self.connections {
            match conn.read() {
                Ok(input) if !input.is_empty() => {
                    if let Some((command, rest)) = take_command(&input) {
                        let pending = PendingCommand {
                            conn_idx: idx,
                            at_room: self.characters[conn.character].in_room,
                            command: lookup_command(COMMANDS, command),
                            arguments: rest.to_string(),
                        };
                        self.pending_commands.push_back(pending);
                    }
                }
                Ok(_) => {
                    log::debug!("Marking linkdead {}: zero length input", conn.addr());
                    self.mark_for_disconnect.push(idx);
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionAborted | ErrorKind::ConnectionReset => {
                            log::debug!("Marking linkdead {}: {}", conn.addr(), e);
                            self.mark_for_disconnect.push(idx);
                        }
                        ErrorKind::WouldBlock => {} // explicitly okay no matter what happens to the catch-all branch
                        _ => log::warn!(
                            "Unexpected input read error from {}: {:?} {}",
                            conn.addr(),
                            e.kind(),
                            e
                        ),
                    }
                }
            }
        }
        for idx in &self.mark_for_disconnect {
            if let Some(_conn) = self.connections.remove(*idx) {
                // TODO: close connection
            }
        }
        self.mark_for_disconnect.clear();
    }

    pub fn run_player_commands(&mut self) {
        let pending_commands = self.pending_commands.split_off(0);
        for pending in pending_commands {
            match pending.command {
                Some(command) => {
                    let _ = (command)(pending.conn_idx, &pending.arguments, self);
                }
                None => {
                    let conn = &mut self.connections[pending.conn_idx];
                    let _ = write!(conn, "I have no idea what that means!");
                }
            }
        }
    }

    pub fn transport_char(&mut self, char_idx: Index, to_room: RoomId) {
        let char = self
            .characters
            .get_mut(char_idx)
            .expect("Unwrapped None character");
        let in_room = self
            .room_chars
            .get_mut(&char.in_room)
            .expect("Unwrapped None room chars");
        if let Some(i) = in_room
            .iter()
            .enumerate()
            .find(|(_, idx)| **idx == char_idx)
            .map(|(i, _)| i)
        {
            in_room.remove(i);
        } else {
            log::warn!("transfer_char: couldn't remove char from {}", char.in_room);
        }
        char.in_room = to_room;
        // Add char index to new room
        if let Some(in_room) = self.room_chars.get_mut(&char.in_room) {
            in_room.push(char_idx);
        } else {
            log::warn!("transfer_char: couldn't move to {}", to_room);
        }
    }
}

fn load_areas() -> (
    Vec<Area>,
    FnvHashMap<CharId, Character>,
    FnvHashMap<ObjectId, ObjectDef>,
    FnvHashMap<RoomId, Room>,
) {
    log::info!("Loading areas");
    let mut areas = Vec::new();
    let mut rooms = FnvHashMap::default();
    let mut object_defs = FnvHashMap::default();
    let mut npcs = FnvHashMap::default();
    // FIXME: daaaaaaang this is ugly
    // TODO: we're gonna iterate over every area name in some text file, rather than load a single area
    match Area::load("default") {
        Ok(mut area_def) => {
            let area_npcs = area_def.extract_npcs();
            let area_objects = area_def.extract_objects();
            let room_defs = area_def.extract_rooms();

            let area = Area::from_prototype(area_def);
            let area_idx = areas.len();
            areas.push(area);
            let area = &mut areas[area_idx];

            for ch in area_npcs {
                if npcs.contains_key(&ch.id()) {
                    log::warn!("Loading areas: clobbered existing NPC {}", ch.id());
                }
                npcs.insert(ch.id(), ch);
            }

            for obj_def in area_objects {
                object_defs.insert(obj_def.id, obj_def);
            }

            for room_def in room_defs {
                let room = Room::from_prototype(room_def, area_idx);
                if rooms.contains_key(&room.id) {
                    log::warn!("Loading areas: clobbered existing room {}", room.id);
                }
                let room_idx = room.id;
                rooms.insert(room.id, room);
                area.rooms.push(room_idx);
            }

            audit_room_exits(&mut rooms);
        }
        Err(e) => log::error!("Error loading area {:?}", e),
    }
    log::info!("Loading areas: success");
    (areas, npcs, object_defs, rooms)
}

fn audit_room_exits(rooms: &mut FnvHashMap<RoomId, Room>) {
    // I should be able to do this in a single iteration of room.values_mut(),
    // and I'm angry that I can't.
    let mut destinations_to_remove = vec![];
    for room in rooms.values() {
        for exit in room.exits.as_ref() {
            if !rooms.contains_key(&exit.to) {
                destinations_to_remove.push(exit.to)
            }
        }
    }
    if !destinations_to_remove.is_empty() {
        for room in rooms.values_mut() {
            for (n, exit) in room.exits.clone().as_ref().iter().enumerate() {
                if destinations_to_remove.contains(&exit.to) {
                    log::warn!(
                        "Loading areas: removed room {}'s exit '{}' to nonexistant {}",
                        room.id,
                        &exit.dir,
                        &exit.to
                    );
                    room.exits.remove(n);
                }
            }
        }
    }
}

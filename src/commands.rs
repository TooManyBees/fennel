mod informational;
mod misc;
mod movement;
mod objects;

use crate::util;
use crate::World;
use generational_arena::Index;
use std::io::Result as IoResult;

pub type CommandFn = fn(Index, &str, &mut World) -> IoResult<()>;

const COMMANDS: &[(&'static str, CommandFn)] = &[
    // Movement commands
    ("north", movement::north),
    ("south", movement::south),
    ("east", movement::east),
    ("west", movement::west),
    ("up", movement::up),
    ("down", movement::down),

    // Common commands
    // ("buy", buy),
    // ("cast", cast),
    // ("exits", exits),
    ("get", objects::get),
    ("inventory", informational::inventory),
    // ("kill", kill),
    // ("fight", fight),
    ("look", informational::look),
    // ("order", order),
    // ("rest", rest),
    // ("sleep", sleep),
    // ("stand", stand),
    // ("tell", tell),
    // ("wield", wield),
    // ("wizhelp", wizhelp),
    ("go", movement::go),

    // Informational commands
    // ("areas", areas),
    // ("commands", commands),
    // ("compare", compare),
    // ("consider", consider),
    // ("credits", credits),
    // ("equipment", equipment),
    // ("examine", examine),
    // ("help", help),
    // ("report", report),
    // ("pagelength", pagelength),
    // ("score", score),
    // ("slist", slist),
    // ("socials", socials),
    // ("time", time),
    // ("weather", weather),
    // ("who", who),
    // ("wizlist", wizlist),

    // Configuration commands
    // ("password", password),
    // ("prmopt", prompt),
    // ("title", title),

    // Communication commands
    // ("chat", chat),
    // (".", chat),
    // ("emote", emote),
    // (",", emote),
    // ("pose", pose),
    // ("reply", reply),
    // ("say", say),
    // ("'", say),
    // ("shout", shout),
    // ("yell", yell),

    // Object manipulation commands
    // ("close", close),
    // ("drink", drink),
    ("drop", objects::drop),
    // ("eat", eat),
    // ("fill", fill),
    // ("give", give),
    // ("hold", hold),
    // ("list", list),
    // ("lock", lock),
    // ("open", open),
    // ("pick", pick),
    // ("put", put),
    // ("quaff", quaff),
    // ("recite", recite),
    // ("remove", remove),
    // ("sell", sell),
    ("take", objects::take),
    // ("unlock", unlock),
    // ("value", value),
    // ("wear", wear),
    // ("zap", zap),

    // Combat commands
    // ("flee", flee),
    // ("rescue", rescue),

    // Misc commands
    // ("follow", follow),
    // ("hide", hide),
    // ("qui", quit_mistake),
    ("quit", misc::quit),
    ("save", misc::save),
    // ("sneak", sneak),
    // ("steal", steal),
    // ("visible", visible),
    // ("wake", wake),
    // ("where", where),
];

pub fn lookup_command(command: &str) -> Option<&'static CommandFn> {
    if command.is_empty() {
        return None;
    }
    let command = command.to_ascii_lowercase();
    util::find_partial(COMMANDS.iter().map(|(k, v)| (k, v)), &command)
}

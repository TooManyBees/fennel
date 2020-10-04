mod informational;
mod misc;
mod movement;

use crate::util::{take_argument};
use crate::World;
use generational_arena::Index;
use std::io::Result as IoResult;

pub type CommandFn = fn(Index, &str, &mut World) -> IoResult<()>;

pub const COMMANDS: &[(&'static str, CommandFn)] = &[
    // Movement commands
    ("north", movement::north),
    ("south", movement::south),
    ("east", movement::east),
    ("west", movement::west),
    ("up", movement::up),
    ("down", movement::down),
    ("go", movement::go),

    // Common commands
    // ("buy", buy),
    // ("cast", cast),
    // ("exits", exits),
    // ("get", get),
    // ("inventory", inventory),
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
    // ("drop", drop),
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
    // ("take", take),
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

pub fn lookup_command<'a, T>(commands: &'a [(&'static str, T)], command: &str) -> Option<&'a T> {
    if command.is_empty() {
        return None;
    }
    let command = command.to_ascii_lowercase();

    let mut found: Option<&'a T> = None;

    for (name, cmd_fn) in commands {
        let is_match = name.starts_with(&command);
        let exact = name.len() == command.len();
        if is_match && exact {
            found = Some(cmd_fn);
            break;
        }
        if is_match && found.is_none() {
            found = Some(cmd_fn);
        }
    }

    found
}

#[cfg(test)]
mod test {
    use super::lookup_command;

    #[derive(Debug, Eq, PartialEq)]
    enum FakeCommand {
        Nobody,
        North,
        Northern,
        There,
        Thorn,
        Throw,
        ThrowAway,
    }
    use FakeCommand::*;

    const COMMANDS: &[(&'static str, FakeCommand)] = &[
        ("north", North),
        ("northern", Northern),
        ("throwaway", ThrowAway),
        ("throw", Throw),
        ("nobody", Nobody),
        ("thorn", Thorn),
        ("there", There),
    ];

    #[test]
    fn find_exact_command() {
        assert_eq!(Some(&North), lookup_command(&COMMANDS, "north"));
    }

    #[test]
    fn prioritize_earlier_matches() {
        assert_eq!(Some(&North), lookup_command(&COMMANDS, "no"));
    }

    #[test]
    fn prioritize_exact_matches() {
        assert_eq!(Some(&Throw), lookup_command(&COMMANDS, "throw"));
    }

    #[test]
    fn find_partial_match() {
        assert_eq!(Some(&There), lookup_command(&COMMANDS, "the"));
    }
}

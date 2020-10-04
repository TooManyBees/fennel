pub fn find_partial<'a, K, V, H>(haystack: H, needle: &str) -> Option<&'a V>
where
    K: 'a + AsRef<str>,
    H: 'a + IntoIterator<Item = &'a (K, V)>,
{
    let mut found: Option<&'a V> = None;

    for (key, val) in haystack.into_iter() {
        let is_match = key.as_ref().starts_with(&needle);
        let exact = key.as_ref().len() == needle.len();
        if is_match && exact {
            found = Some(val);
            break;
        }
        if is_match && found.is_none() {
            found = Some(val);
        }
    }

    found
}

#[cfg(test)]
mod test {
    use super::find_partial;

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
        assert_eq!(Some(&North), find_partial(COMMANDS, "north"));
    }

    #[test]
    fn prioritize_earlier_matches() {
        assert_eq!(Some(&North), find_partial(COMMANDS, "no"));
    }

    #[test]
    fn prioritize_exact_matches() {
        assert_eq!(Some(&Throw), find_partial(COMMANDS, "throw"));
    }

    #[test]
    fn find_partial_match() {
        assert_eq!(Some(&There), find_partial(COMMANDS, "the"));
    }
}

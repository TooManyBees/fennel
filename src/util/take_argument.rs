// A quote-aware function to separate the first keyword from a string

#[derive(Copy, Clone, Debug)]
enum ParseState {
    Begin,
    Unquoted,
    // Whitespace,
    DoubleQuote,
    SingleQuote,
    // Can expand into non-english quotes here too
}

pub fn take_command(input: &str) -> Option<(&str, &str)> {
    assert!(
        input.ends_with('\n') || input.ends_with('\r'),
        "take_argument input string must end with a CR or LF"
    );

    let partition = input
        .char_indices()
        .skip_while(|(_, c)| c.is_ascii_whitespace())
        .skip_while(|(_, c)| !c.is_ascii_whitespace())
        .next();

    match partition {
        // Some((_, ch)) if ch.is_ascii_whitespace() => None,
        Some((idx, _)) => {
            let (command, rest) = input.split_at(idx);
            Some((command.trim(), rest))
        }
        None => None,
    }
}

pub fn take_argument(input: &str) -> Option<(&str, &str)> {
    assert!(
        input.ends_with('\n') || input.ends_with('\r'),
        "take_argument input string must end with a CR or LF"
    );
    let mut start_of_keyword = 0;
    let mut end_of_keyword = 0;
    let mut trim_off_remainder = 0;
    let mut state = ParseState::Begin;
    for (idx, ch) in input.char_indices() {
        end_of_keyword = idx;
        match (ch, state) {
            ('\n', ParseState::Begin) | ('\r', ParseState::Begin) => {
                break;
            }
            (_, ParseState::Begin) if ch.is_ascii_whitespace() => {}
            ('"', ParseState::Begin) => {
                state = ParseState::DoubleQuote;
                start_of_keyword = idx + ch.len_utf8();
            }
            ('\'', ParseState::Begin) => {
                state = ParseState::SingleQuote;
                start_of_keyword = idx + ch.len_utf8();
            }
            (_, ParseState::Begin) => {
                state = ParseState::Unquoted;
                start_of_keyword = idx;
            }
            ('"', ParseState::DoubleQuote) => {
                trim_off_remainder = ch.len_utf8();
                break;
            }
            ('\'', ParseState::SingleQuote) => {
                trim_off_remainder = ch.len_utf8();
                break;
            }
            (_, ParseState::Unquoted) if ch.is_ascii_whitespace() => {
                break;
            }
            _ => {}
        }
    }
    if let ParseState::Begin = state {
        return None;
    }
    debug_assert!(start_of_keyword <= end_of_keyword);
    let (kw, rest) = input.split_at(end_of_keyword);
    let kw = &kw[start_of_keyword..];
    let rest = &rest[trim_off_remainder..];
    Some((kw, rest))
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use super::{take_command, take_argument};

    #[test]
    fn command_no_arguments() {
        let input = "command\r\n";
        let command = take_command(input).map(|(command, _)| command);
        assert_eq!(command, Some("command"));
    }

    #[test]
    fn command_with_arguments() {
        let input = "command whatever else\r\n";
        let (command, rest) = take_command(input).unwrap();
        assert_eq!(command, "command");
        assert_eq!(rest, " whatever else\r\n");
    }

    #[test]
    fn take_command_whitespace() {
        let input = "  \r\n";
        assert_eq!(None, take_command(input));
    }

    #[test]
    fn take_command_empty() {
        let input = "\r\n";
        assert_eq!(None, take_command(input));
    }

    macro_rules! test_argument {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let input = $input;
                let kw = take_argument(input).map(|(kw, _rest)| kw);
                assert_eq!(kw, $expected);
            }
        };
    }

    test_argument!(single_word, "hello\r\n", Some("hello"));
    test_argument!(surrounding_whitespace, "    hello  \r\n", Some("hello"));
    test_argument!(single_quote, " 'up tree'\r\n", Some("up tree"));
    test_argument!(double_quote, " \"up tree\"\r\n", Some("up tree"));
    test_argument!(nested_single_quotes, " \"barry's account book\" barry\r\n", Some("barry's account book"));
    test_argument!(nested_double_quotes, " 'very \"fancy\" book' barry\r\n", Some("very \"fancy\" book"));
    test_argument!(all_whitespace, "    \r\n", None);
    test_argument!(empty_input, "\r\n", None);

    #[test]
    fn repeated() {
        let input = "put \"important thing\" in \'big box\'  \r\n";
        let (kw1, rest) = take_argument(input).unwrap();
        let (kw2, rest) = take_argument(rest).unwrap();
        let (kw3, rest) = take_argument(rest).unwrap();
        let (kw4, _rest) = take_argument(rest).unwrap();
        assert_eq!(&[kw1, kw2, kw3, kw4], &["put", "important thing", "in", "big box"]);
    }
}

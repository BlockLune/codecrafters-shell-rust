#[derive(PartialEq)]
enum TokenizerState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token: String = String::new();
    let mut state = TokenizerState::Normal;
    let mut is_escaping = false;

    for ch in input.chars() {
        if is_escaping {
            token.push(ch);
            is_escaping = false;
            continue;
        }

        match ch {
            '\'' => match state {
                TokenizerState::Normal => state = TokenizerState::InSingleQuote,
                TokenizerState::InSingleQuote => state = TokenizerState::Normal,
                TokenizerState::InDoubleQuote => token.push('\''),
            },
            '\"' => match state {
                TokenizerState::Normal => state = TokenizerState::InDoubleQuote,
                TokenizerState::InSingleQuote => token.push('\"'),
                TokenizerState::InDoubleQuote => state = TokenizerState::Normal,
            },
            '\\' => match state {
                TokenizerState::Normal | TokenizerState::InDoubleQuote => is_escaping = true,
                TokenizerState::InSingleQuote => token.push('\\'),
            },
            ' ' => match state {
                TokenizerState::Normal => {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
                TokenizerState::InSingleQuote | TokenizerState::InDoubleQuote => token.push(' '),
            },
            // pipe splits at tokenizer level so the state machine can distinguish
            // literal | inside quotes from the pipeline operator
            '|' => match state {
                TokenizerState::Normal => {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                    tokens.push("|".to_string());
                }
                _ => token.push('|'),
            },
            other_ch => {
                token.push(other_ch);
            }
        }
    }

    if state != TokenizerState::Normal {
        return Err(String::from("unclosed"));
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_multiple_whitespaces_in_single_quotes() {
        assert_eq!(
            tokenize("echo 'hello    world'").unwrap(),
            vec!["echo", "hello    world"]
        );
    }

    #[test]
    fn test_tokenizer_multiple_whitespaces_normal() {
        assert_eq!(
            tokenize("echo hello    world").unwrap(),
            vec!["echo", "hello", "world"]
        );
    }

    #[test]
    fn test_tokenizer_adjacent_single_quoted_strings() {
        assert_eq!(
            tokenize("echo 'hello''world'").unwrap(),
            vec!["echo", "helloworld"]
        );
    }

    #[test]
    fn test_tokenizer_empty_single_quotes() {
        assert_eq!(
            tokenize("echo hello''world").unwrap(),
            vec!["echo", "helloworld"]
        );
    }

    #[test]
    fn test_tokenizer_multiple_whitespaces_in_double_quotes() {
        assert_eq!(
            tokenize(r#"echo "hello    world""#).unwrap(),
            vec!["echo", "hello    world"]
        )
    }

    #[test]
    fn test_tokenizer_adjacent_double_quoted_strings() {
        assert_eq!(
            tokenize(r#"echo "hello""world""#).unwrap(),
            vec!["echo", "helloworld"]
        )
    }

    #[test]
    fn test_tokenizer_double_quoted_and_unquoted_strings_next_to_each_other() {
        assert_eq!(
            tokenize(r#"echo "hello"world"#).unwrap(),
            vec!["echo", "helloworld"]
        )
    }

    #[test]
    fn test_tokenizer_separate_arguments_in_double_quotes() {
        assert_eq!(
            tokenize(r#"echo "hello" "world""#).unwrap(),
            vec!["echo", "hello", "world"]
        )
    }

    #[test]
    fn test_tokenizer_single_quotes_in_double_quotes() {
        assert_eq!(
            tokenize(r#"echo "shell's test""#).unwrap(),
            vec!["echo", "shell's test"]
        )
    }

    #[test]
    fn test_tokenizer_multiple_escaped_spaces() {
        assert_eq!(
            tokenize(r#"echo three\ \ \ spaces"#).unwrap(),
            vec!["echo", "three   spaces"]
        );
    }

    #[test]
    fn test_tokenizer_escaped_space_and_delimiters() {
        assert_eq!(
            tokenize(r#"echo before\      after"#).unwrap(),
            vec!["echo", "before ", "after"]
        );
    }

    #[test]
    fn test_tokenizer_escaped_n_character() {
        assert_eq!(
            tokenize(r#"echo test\nexample"#).unwrap(),
            vec!["echo", "testnexample"]
        );
    }

    #[test]
    fn test_tokenizer_escaped_backslash() {
        assert_eq!(
            tokenize(r#"echo hello\\world"#).unwrap(),
            vec!["echo", r#"hello\world"#]
        );
    }

    #[test]
    fn test_tokenizer_escaped_single_quotes() {
        assert_eq!(
            tokenize(r#"echo \'hello\'"#).unwrap(),
            vec!["echo", "'hello'"]
        );
    }

    #[test]
    fn test_tokenizer_single_quotes_literal_backslashes() {
        assert_eq!(
            tokenize(r#"echo 'shell\\\nscript'"#).unwrap(),
            vec!["echo", r#"shell\\\nscript"#]
        );
    }

    #[test]
    fn test_tokenizer_single_quotes_internal_double_quotes() {
        assert_eq!(
            tokenize(r#"echo 'example\"test'"#).unwrap(),
            vec!["echo", r#"example\"test"#]
        );
    }

    #[test]
    fn test_tokenizer_double_quotes_with_backslashes() {
        assert_eq!(
            tokenize(r#"echo "just'one'\\n'backslash""#).unwrap(),
            vec!["echo", r#"just'one'\n'backslash"#]
        );
    }

    #[test]
    fn test_tokenizer_double_quotes_mixed_with_raw_text() {
        assert_eq!(
            tokenize(r#"echo "inside\"literal_quote."outside\""#).unwrap(),
            vec!["echo", r#"inside"literal_quote.outside""#]
        );
    }
}

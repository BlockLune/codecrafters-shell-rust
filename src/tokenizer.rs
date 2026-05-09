#[derive(PartialEq)]
enum TokenizerState {
    Normal,
    InSingleQuote,
    InDoubleQuote,
}

pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut state = TokenizerState::Normal;
    let mut token: String = String::new();

    for ch in input.chars() {
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
            ' ' => match state {
                TokenizerState::Normal => {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
                TokenizerState::InSingleQuote | TokenizerState::InDoubleQuote => token.push(' '),
            },
            other_ch => {
                token.push(other_ch);
            }
        }
    }

    if state != TokenizerState::Normal {
        return Err(String::from("unclosed"));
    }

    tokens.push(token.clone());

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
            tokenize("echo \"hello    world\"").unwrap(),
            vec!["echo", "hello    world"]
        )
    }

    #[test]
    fn test_tokenizer_adjacent_double_quoted_strings() {
        assert_eq!(
            tokenize("echo \"hello\"\"world\"").unwrap(),
            vec!["echo", "helloworld"]
        )
    }

    #[test]
    fn test_tokenizer_double_quoted_and_unquoted_strings_next_to_each_other() {
        assert_eq!(
            tokenize("echo \"hello\"world").unwrap(),
            vec!["echo", "helloworld"]
        )
    }

    #[test]
    fn test_tokenizer_separate_arguments_in_double_quotes() {
        assert_eq!(
            tokenize("echo \"hello\" \"world\"").unwrap(),
            vec!["echo", "hello", "world"]
        )
    }

    #[test]
    fn test_tokenizer_single_quotes_in_double_quotes() {
        assert_eq!(
            tokenize("echo \"shell\'s test\"").unwrap(),
            vec!["echo", "shell\'s test"]
        )
    }
}

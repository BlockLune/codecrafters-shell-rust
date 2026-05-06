#[derive(PartialEq)]
enum TokenizerState {
    Normal,
    InSingleQuote,
}

pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut state = TokenizerState::Normal;
    let mut token: String = String::new();

    for ch in input.chars() {
        match ch {
            '\'' => {
                state = match state {
                    TokenizerState::Normal => TokenizerState::InSingleQuote,
                    TokenizerState::InSingleQuote => TokenizerState::Normal,
                }
            }
            ' ' => match state {
                TokenizerState::Normal => {
                    if !token.is_empty() {
                        tokens.push(token.clone());
                        token.clear();
                    }
                }
                TokenizerState::InSingleQuote => token.push(' '),
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
}

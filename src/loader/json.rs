use std::{collections::HashMap, iter::Peekable, str::Chars};

use crate::{log_error, log_info};

#[derive(Debug)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Number(Number),
    String(String),
    Boolean(bool),
    Null,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
enum Token {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    String(String),
    Number(Number),
    Boolean(bool),
    Null,
}

pub fn parse(input: &str) -> Option<HashMap<String, Value>> {
    log_info!("Loading JSON");
    if input.is_empty() {
        log_error!("Invalid JSON, tried to parse empty JSON string");
        return None;
    }
    let tokens = lex(input);
    log_info!("{:?}", tokens);
    return parse_tokens(tokens);
}

fn parse_tokens(tokens: Vec<Token>) -> Option<HashMap<String, Value>> {
    let mut iter = tokens.iter().peekable();

    let start_time = std::time::Instant::now();
    if let Some(value) = parse_value(&mut iter, 0) {
        log_info!("JSON parser took {} us", start_time.elapsed().as_micros());
        match value {
            Value::Object(object) => return Some(object),
            Value::Array(_) => {
                let mut object = HashMap::new();
                object.insert("".into(), value);
                return Some(object);
            }
            _ => return None,
        }
    } else {
        return None;
    }
}

fn parse_value<'a, I>(iter: &mut Peekable<I>, depth: usize) -> Option<Value>
where
    I: Iterator<Item = &'a Token>,
{
    if depth > 64 {
        log_error!("Invalid JSON, depth > 64");
        return None;
    }

    let value = match iter.peek() {
        Some(Token::LBrace) => parse_object(iter, depth),
        Some(Token::LBracket) => parse_array(iter, depth),
        Some(Token::String(string)) => {
            iter.next();
            Some(Value::String(string.clone()))
        }
        Some(Token::Number(number)) => {
            iter.next();
            Some(Value::Number(*number))
        }
        Some(Token::Boolean(boolean)) => {
            iter.next();
            Some(Value::Boolean(*boolean))
        }
        Some(Token::Null) => {
            iter.next();
            Some(Value::Null)
        }
        _ => None,
    };

    return value;
}

fn parse_object<'a, I>(iter: &mut Peekable<I>, depth: usize) -> Option<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let mut object: HashMap<String, Value> = HashMap::new();

    iter.next();
    loop {
        match iter.peek() {
            Some(Token::RBrace) => {
                iter.next();
                break;
            }
            Some(Token::String(name)) => {
                iter.next();
                match iter.peek() {
                    Some(Token::Colon) => {
                        iter.next();
                        if let Some(value) = parse_value(iter, depth + 1) {
                            object.insert(name.clone(), value);
                            match iter.peek() {
                                Some(Token::RBrace) => {
                                    iter.next();
                                    break;
                                }
                                Some(Token::Comma) => {
                                    iter.next();
                                    continue;
                                }
                                None => break,
                                token @ _ => {
                                    log_error!(
                                        "Invalid JSON, expected RBrace or Comma but found '{:?}'",
                                        token
                                    );
                                    return None;
                                }
                            }
                        }
                    }
                    None => break,
                    token @ _ => {
                        log_error!("Invalid JSON, expected Colon but found '{:?}'", token);
                        return None;
                    }
                }
            }
            None => break,
            token @ _ => {
                log_error!(
                    "Invalid JSON, expected RBrace or String but found '{:?}'",
                    token
                );
                return None;
            }
        }
    }

    return Some(Value::Object(object));
}

fn parse_array<'a, I>(iter: &mut Peekable<I>, depth: usize) -> Option<Value>
where
    I: Iterator<Item = &'a Token>,
{
    let mut array: Vec<Value> = vec![];

    iter.next();
    loop {
        match iter.peek() {
            Some(Token::RBracket) => {
                iter.next();
                break;
            }
            Some(Token::String(_))
            | Some(Token::Number(_))
            | Some(Token::Boolean(_))
            | Some(Token::Null)
            | Some(Token::LBrace)
            | Some(Token::LBracket) => {
                if let Some(value) = parse_value(iter, depth + 1) {
                    array.push(value);
                    match iter.peek() {
                        Some(Token::Comma) => {
                            iter.next();
                            continue;
                        }
                        _ => {
                            iter.next();
                            break;
                        }
                    }
                }
            }
            token @ _ => {
                log_error!("Invalid JSON, expected a value but found '{:?}'", token);
                return None;
            }
        }
    }

    return Some(Value::Array(array));
}

fn lex(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.peek() {
        match c {
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RBracket);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '"' => {
                tokens.push(Token::String(get_string(&mut chars)));
            }
            _ if c.is_ascii_digit() || *c == '-' => {
                if let Some(number) = get_number(&mut chars) {
                    tokens.push(Token::Number(number))
                }
            }
            't' | 'f' => {
                if let Some(boolean) = get_boolean(&mut chars) {
                    tokens.push(Token::Boolean(boolean));
                }
            }
            'n' => {
                if get_null(&mut chars) {
                    tokens.push(Token::Null);
                }
            }
            _ if c.is_ascii_whitespace() => {
                chars.next();
            }
            _ => {
                log_error!("Invalid JSON character, found {}", c);
                break;
            }
        }
    }

    return tokens;
}

fn get_null(chars: &mut Peekable<Chars>) -> bool {
    let mut string = String::new();
    while let Some(c) = chars.peek() {
        match c {
            'a'..='z' => {
                string.push(*c);
                chars.next();
            }
            _ => break,
        }
    }
    return match string.as_str() {
        "null" => true,
        _ => false,
    };
}

fn get_boolean(chars: &mut Peekable<Chars>) -> Option<bool> {
    let mut string = String::new();
    while let Some(c) = chars.peek() {
        match c {
            'a'..='z' => {
                string.push(*c);
                chars.next();
            }
            _ => break,
        }
    }
    return match string.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    };
}

fn get_string(chars: &mut Peekable<Chars>) -> String {
    chars.next();
    chars.take_while(|c| *c != '"').into_iter().collect()
}

fn get_number(chars: &mut Peekable<Chars>) -> Option<Number> {
    let mut number_string = String::new();
    while let Some(c) = chars.peek() {
        match c {
            '-' | '0'..='9' | '.' => {
                number_string.push(*c);
                chars.next();
            }
            _ => break,
        }
    }

    if number_string.contains('.') {
        if let Ok(float) = str::parse::<f64>(&number_string) {
            return Some(Number::Float(float));
        } else {
            log_error!("Invalid float: '{}'", number_string);
            return None;
        }
    } else {
        if let Ok(integer) = str::parse::<i64>(&number_string) {
            return Some(Number::Integer(integer));
        } else {
            log_error!("Invalid integer: '{}'", number_string);
            return None;
        }
    }
}

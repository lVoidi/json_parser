use std::collections::HashMap;

#[derive(Debug)]
enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
enum Token {
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Colon,          // :
    Comma,          // ,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    // Iterador sobre los caracteres de la entrada
    // Peekable para poder mirar el siguiente caracter
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Ignorar espacios en blanco
            c if c.is_whitespace() => {
                chars.next();
            },
            // Símbolos simples
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            },
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            },
            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            },
            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            },
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            },
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            },
            // Strings
            '"' => {
                chars.next(); // Consumir la comilla inicial
                let mut string = String::new();
                
                while let Some(&c) = chars.peek() {
                    match c {
                        '"' => {
                            chars.next();
                            break;
                        },
                        '\\' => {
                            chars.next(); // Consumimos el caracter de escape
                            if let Some(next_char) = chars.next() {
                                match next_char {
                                    '"' | '\\' | '/' => string.push(next_char),
                                    'b' => string.push('\x08'),
                                    'f' => string.push('\x0c'),
                                    'n' => string.push('\n'),
                                    'r' => string.push('\r'),
                                    't' => string.push('\t'),
                                    _ => return Err("Secuencia de escape inválida".to_string()),
                                }
                            }
                        },
                        _ => {
                            string.push(c);
                            chars.next();
                        }
                    }
                }
                tokens.push(Token::String(string));
            },
            // Números
            c if c.is_digit(10) || c == '-' => {
                let mut number = String::new();
                if c == '-' {
                    number.push(c);
                    chars.next();
                }
                
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                        number.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                match number.parse::<f64>() {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err("Número inválido".to_string()),
                }
            },
            // Valores literales (true, false, null)
            't' => {
                let mut rest = String::new();
                for _ in 0..4 {
                    if let Some(c) = chars.next() {
                        rest.push(c);
                    }
                }
                if rest == "true" {
                    tokens.push(Token::Boolean(true));
                } else {
                    return Err("Token inválido: esperaba 'true'".to_string());
                }
            },
            'f' => {
                let mut rest = String::new();
                for _ in 0..5 {
                    if let Some(c) = chars.next() {
                        rest.push(c);
                    }
                }
                if rest == "false" {
                    tokens.push(Token::Boolean(false));
                } else {
                    return Err("Token inválido: esperaba 'false'".to_string());
                }
            },
            'n' => {
                let mut rest = String::new();
                for _ in 0..4 {
                    if let Some(c) = chars.next() {
                        rest.push(c);
                    }
                }
                if rest == "null" {
                    tokens.push(Token::Null);
                } else {
                    return Err("Token inválido: esperaba 'null'".to_string());
                }
            },
            _ => return Err(format!("Carácter inesperado: {}", c)),
        }
    }
    
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1)
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek().ok_or("Fin inesperado de entrada")? {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::String(_) => {
                let token = self.advance().unwrap();
                if let Token::String(s) = token {
                    Ok(JsonValue::String(s.clone()))
                } else {
                    unreachable!()
                }
            },
            Token::Number(n) => {
                let num = *n; // Copiamos el valor antes de advance
                self.advance();
                Ok(JsonValue::Number(num))
            },
            Token::Boolean(b) => {
                let bool_val = *b; // Copiamos el valor antes de advance
                self.advance();
                Ok(JsonValue::Boolean(bool_val))
            },
            Token::Null => {
                self.advance();
                Ok(JsonValue::Null)
            },
            _ => Err("Token inesperado".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Consumir '{'
        let mut map = HashMap::new();

        while let Some(token) = self.peek() {
            if matches!(token, Token::RightBrace) {
                self.advance(); // Consumir '}'
                return Ok(JsonValue::Object(map));
            }

            // Parsear la key (debe ser un string)
            let key = if let Token::String(s) = self.advance().ok_or("Se esperaba una key")? {
                s.clone()
            } else {
                return Err("La key debe ser un string".to_string());
            };

            // Esperar ':'
            if !matches!(self.advance(), Some(Token::Colon)) {
                return Err("Se esperaba ':'".to_string());
            }

            // Parsear el valor
            let value = self.parse_value()?;
            map.insert(key, value);

            // Verificar si hay más elementos
            if let Some(token) = self.peek() {
                match token {
                    Token::Comma => {
                        self.advance(); // Consumir ','
                        continue;
                    },
                    Token::RightBrace => continue,
                    _ => return Err("Se esperaba ',' o '}'".to_string()),
                }
            }
        }

        Err("Objeto no cerrado correctamente".to_string())
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Consumir '['
        let mut array = Vec::new();

        while let Some(token) = self.peek() {
            if matches!(token, Token::RightBracket) {
                self.advance(); // Consumir ']'
                return Ok(JsonValue::Array(array));
            }

            array.push(self.parse_value()?);

            if let Some(token) = self.peek() {
                match token {
                    Token::Comma => {
                        self.advance(); // Consumir ','
                        continue;
                    },
                    Token::RightBracket => continue,
                    _ => return Err("Se esperaba ',' o ']'".to_string()),
                }
            }
        }

        Err("Array no cerrado correctamente".to_string())
    }
}

fn main() {
    let json_str = r#"
    {
        "nombre": "Juan",
        "edad": 30,
        "activo": true,
        "hobbies": ["programar", "leer"],
        "direccion": {
            "calle": "Principal",
            "numero": 123
        }
    }
    "#;

    match tokenize(json_str) {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(value) => println!("JSON parseado: {:#?}", value),
                Err(e) => println!("Error al parsear: {}", e),
            }
        },
        Err(e) => println!("Error al tokenizar: {}", e),
    }
}

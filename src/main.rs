use std::{f32::consts::E, fmt::Error};

struct ParseLogicalLine {
    indent: usize,
    statement: ParseStatement,
}

enum ParseStatement {
    Definition { key: String, character: Character },
    Label { key: String },
    Dialogue { character_key: String, text: String },
    Menu {},
    Choice { text: String },
    Jump { key: String },
}

struct Character {
    name: String,
    color: String,
}

fn main() {
    let script = std::fs::read_to_string("script.rpy").unwrap();
    let mut logical_lines = Vec::<ParseLogicalLine>::new();
    let mut look_for_keys = Vec::<String>::new();

    logical_lines.push(ParseLogicalLine {
        indent: 0,
        statement: ParseStatement::Definition {
            key: "".to_string(),
            character: Character {
                name: "".to_string(),
                color: "".to_string(),
            },
        },
    });

    for line in script.lines() {
        match parse_line(line.to_string(), &mut look_for_keys) {
            Ok(logical_line) => logical_lines.push(logical_line),
            Err(_) => (println!("Invalid line: {}", line)),
        }
    }

    for line in logical_lines {
        let statement = line.statement;
        print!("{}", " ".repeat(line.indent));
        match statement {
            ParseStatement::Definition { key, character } => {
                println!("define {}: {}", key, character.name);
            }
            ParseStatement::Label { key } => {
                println!("Label: {}", key);
            }
            ParseStatement::Dialogue {
                character_key,
                text,
            } => {
                println!("{}: {}", character_key, text);
            }
            ParseStatement::Menu {} => {
                println!("Menu");
            }
            ParseStatement::Choice { text } => {
                println!("Choice: {}", text);
            }
            ParseStatement::Jump { key } => {
                println!("Jump: {}", key);
            }
        }
    }
}

fn parse_line(line: String, look_for_keys: &mut Vec<String>) -> Result<ParseLogicalLine, &'static str> {
    let line_trim = line.trim();
    if line_trim.starts_with("define") {
        let line_new = line_trim.replace("define", "");
        let line_split: Vec<String> = line_new.split("=").map(|x| x.to_string()).collect();

        let key = line_split[0].trim().to_string();

        let name_first_quote = line_split[1].find("\"").unwrap();
        let name_last_quote = line_split[1].rfind("\"").unwrap();
        let name = line_split[1][name_first_quote + 1..name_last_quote].to_string();

        let color_first_quote = line_split[2].find("\"").unwrap();
        let color_last_quote = line_split[2].rfind("\"").unwrap();
        let color = line_split[2][color_first_quote + 1..color_last_quote].to_string();

        let character = Character {
            name: name,
            color: color,
        };

        look_for_keys.push(key.clone());
        return Ok(ParseLogicalLine {
            indent: line.find("define").unwrap(),
            statement: ParseStatement::Definition {
                key: key,
                character: character,
            },
        });
    } else if line_trim.starts_with("label") {
        let line_new = line_trim.replace("label", "").trim().to_string();
        let key = line_new.replace(":", "").trim().to_string();
        return Ok(ParseLogicalLine {
            indent: line.find("label").unwrap(),
            statement: ParseStatement::Label { key: key },
        });
    } else if line_trim.starts_with("\"") {
        let text = line_trim.trim().to_string();
        if line.ends_with(":") {
            return Ok(ParseLogicalLine {
                indent: line.find("\"").unwrap(),
                statement: ParseStatement::Choice { text: text },
            });
        }
        return Ok(ParseLogicalLine {
            indent: line.find("\"").unwrap(),
            statement: ParseStatement::Dialogue {
                character_key: "".to_string(),
                text: text,
            },
        });
    } else if line_trim.starts_with("menu") {
        return Ok(ParseLogicalLine {
            indent: line.find("menu").unwrap(),
            statement: ParseStatement::Menu {},
        });
    } else if line_trim.starts_with("jump") {
        let line_new = line_trim.replace("jump", "").trim().to_string();
        let key = line_new.replace(":", "").trim().to_string();
        return Ok(ParseLogicalLine {
            indent: line.find("jump").unwrap(),
            statement: ParseStatement::Jump { key: key },
        });
    } else {
        for key in look_for_keys.iter() {
            if line_trim.starts_with(format!("{} ", key).as_str()) {
                let text = line_trim
                    .split(" ")
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join(" ");
                return Ok(ParseLogicalLine {
                    indent: line.find(key).unwrap(),
                    statement: ParseStatement::Dialogue {
                        character_key: key.clone(),
                        text: text,
                    },
                });
            }
        }
    }
    return Err("Invalid line");
}

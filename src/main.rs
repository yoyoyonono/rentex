struct LogicalLine {
    indent: usize,
    statement: Statement,
}

enum Statement {
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
    let mut logical_lines = Vec::<LogicalLine>::new();
    let mut look_for_keys = Vec::<String>::new();

    logical_lines.push(LogicalLine{indent: 0, statement: Statement::Definition { key: "".to_string(), character: Character { name: "".to_string(), color: "".to_string() } }});

    for line in script.lines() {
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

            logical_lines.push(LogicalLine{indent: line.find("define").unwrap(),
                statement: Statement::Definition {
                key: key.clone(),
                character: character,
            }});
            look_for_keys.push(key);
        } else if line_trim.starts_with("label") {
            let line_new = line_trim.replace("label", "").trim().to_string();
            let key = line_new.replace(":", "").trim().to_string();
            logical_lines.push(LogicalLine{indent: line.find("label").unwrap(),
                statement: Statement::Label {
                key: key,
            }});
        } else if line_trim.starts_with("\"") {
            let text = line_trim.trim().to_string();
            if line.ends_with(":") {
                logical_lines.push(LogicalLine{indent: line.find("\"").unwrap(),
                    statement: Statement::Choice {
                    text: text,
                }});
                continue;
            }
            logical_lines.push(LogicalLine{indent: line.find("\"").unwrap(),
                statement: Statement::Dialogue {
                character_key: "".to_string(),
                text: text,
            }});
        } else if line_trim.starts_with("menu") {
            logical_lines.push(LogicalLine{indent: line.find("menu").unwrap(),
                statement: Statement::Menu {}});
        } else if line_trim.starts_with("jump") {
            let line_new = line_trim.replace("jump", "").trim().to_string();
            let key = line_new.replace(":", "").trim().to_string();
            logical_lines.push(LogicalLine{indent: line.find("jump").unwrap(),
                statement: Statement::Jump {
                key: key,
            }});
        }
        else {
            for key in look_for_keys.iter() {
                if line_trim.starts_with(format!("{} ", key).as_str()) {
                    let text = line_trim.split(" ").skip(1).collect::<Vec<&str>>().join(" ");
                    logical_lines.push(LogicalLine{indent: line.find(key).unwrap(),
                        statement: Statement::Dialogue {
                        character_key: key.clone(),
                        text: text,
                    }});
                }
            }
        }
    }

    for line in logical_lines {
        let statement = line.statement;
        print!("{}", " ".repeat(line.indent));
        match statement {
            Statement::Definition { key, character } => {
                println!("define {}: {}", key, character.name);
            }
            Statement::Label { key } => {
                println!("Label: {}", key);
            }
            Statement::Dialogue { character_key, text} => {
                println!("{}: {}", character_key, text);
            }
            Statement::Menu {} => {
                println!("Menu");
            }
            Statement::Choice { text } => {
                println!("Choice: {}", text);
            }
            Statement::Jump { key } => {
                println!("Jump: {}", key);
            }
        }
    }
}

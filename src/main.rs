use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ParseLogicalLine {
    indent: usize,
    statement: ParseStatement,
}

#[derive(Debug, Clone)]
enum ParseStatement {
    Definition { key: String, character: Character },
    Label { key: String },
    Dialogue { character_key: String, text: String },
    Menu {},
    Choice { text: String },
    Jump { key: String },
    End {},
}

#[derive(Debug, Clone)]
struct Character {
    name: String,
    color: String,
}

#[derive(Debug, Clone)]
struct Page {
    index: usize,
    label: Option::<String>,
    content: Page_Content,
    unconditional_jump: Option::<String>,
}

#[derive(Debug, Clone)]
enum Page_Content {
    Dialogue {
        character_name: String,
        text: String,
    },
    Menu {
        character_name: String,
        text: String,
        choices: Vec<MenuChoice>,
    },
}

#[derive(Debug, Clone)]
struct MenuChoice {
    text: String,
    jump_key: String,
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

    for line in logical_lines.clone() {
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
            ParseStatement::End {} => {
                println!("End");
            }
        }
    }

    let pages = traverse_game(logical_lines);

    println!("Pages: {:#?}", pages);

    let latex = latex_output(pages);

    std::fs::write("out.tex", latex).unwrap();
}

fn parse_line(
    line: String,
    look_for_keys: &mut Vec<String>,
) -> Result<ParseLogicalLine, &'static str> {
    let line_trim = line.trim();
    if line_trim.starts_with("define") && line_trim.contains("Character") {
        let line_new = line_trim.replace("define", "");
        let line_split: Vec<String> = line_new.split("=").map(|x| x.to_string()).collect();

        let key = line_split[0].trim().to_string();

        let name_first_quote = line_split[1].find("\"").unwrap();
        let name_last_quote = line_split[1].rfind("\"").unwrap();
        let name = line_split[1][name_first_quote + 1..name_last_quote].to_string();

        let color = if line_trim.contains("color") {
            let color_first_quote = line_split[2].find("\"").unwrap();
            let color_last_quote = line_split[2].rfind("\"").unwrap();
            line_split[2][color_first_quote + 1..color_last_quote].to_string()
        } else {
            "".to_string()
        };

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
                statement: ParseStatement::Choice { text: clean_up_text(text) },
            });
        }
        return Ok(ParseLogicalLine {
            indent: line.find("\"").unwrap(),
            statement: ParseStatement::Dialogue {
                character_key: "".to_string(),
                text: clean_up_text(text),
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
    } else if line_trim.starts_with("return") {
        return Ok(ParseLogicalLine {
            indent: line.find("return").unwrap(),
            statement: ParseStatement::End {},
        });
    } else if line_trim.starts_with("$ speak") {
        // Example line 
        // $ speak(NICOLE, "Long story...")
        let line_new = line_trim.replace("$ speak(", "").trim().to_string();        
        let line_split: Vec<String> = line_new.splitn(2, ",").map(|x| x.to_string()).collect();
        let key = line_split[0].trim().to_string();
        let first_quote = line_split[1].find("\"").unwrap();
        let last_quote = line_split[1].rfind("\"").unwrap();
        let text = line_split[1][first_quote + 1..last_quote].to_string();
        return Ok(ParseLogicalLine {
            indent: line.find("$ speak").unwrap(),
            statement: ParseStatement::Dialogue {
                character_key: key,
                text: clean_up_text(text),
            },
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
                        text: clean_up_text(text),
                    },
                });
            }
        }
    }
    return Err("Invalid line");
}

fn traverse_game(logical_lines: Vec<ParseLogicalLine>) -> Vec<Page> {
    let mut pages = Vec::<Page>::new();
    let mut characters = HashMap::<String, Character>::new();

    // push title page

    pages.push(Page {
        index: 0,
        label: None,
        content: Page_Content::Dialogue {
            character_name: "".to_string(),
            text: "Title Page".to_string(),
        },
        unconditional_jump: None,
    });

    // define characters
    for line in logical_lines.clone() {
        let statement = &line.statement;
        match statement {
            ParseStatement::Definition { key, character } => {
                characters.insert(key.clone(), character.clone());
            }
            _ => (),
        }
    }

    // find start label
    let label_start_index = find_label_index(logical_lines.clone(), "start".to_string());
    println!("Start index: {}", label_start_index);

    let mut current_index = label_start_index + 1;
    let mut next_has_label = false;
    let mut next_label = "".to_string();
    loop {
        let line = &logical_lines[current_index];
        let statement = &line.statement;
        match statement {
            ParseStatement::Dialogue {
                character_key,
                text,
            } => {
                let mut label = None;
                if next_has_label {
                    label = Some(next_label.clone());
                    next_has_label = false;
                }
                pages.push(Page {
                    index: current_index,
                    label: label,
                    content: Page_Content::Dialogue {
                        character_name: characters.get(character_key).unwrap().name.clone(),
                        text: text.clone(),
                    },
                    unconditional_jump: None,
                });
                current_index += 1;
            }
            ParseStatement::Menu {} => {
                let mut label = None;
                if next_has_label {
                    label = Some(next_label.clone());
                    next_has_label = false;
                }
                let mut choices = Vec::<MenuChoice>::new();

                let mut character_name = "".to_string();
                let mut character_text: String = "".to_string();

                current_index = current_index + 1;
                loop {
                    let line = &logical_lines[current_index];
                    let statement = &line.statement;
                    match statement {
                        ParseStatement::Choice { text } => {
                            choices.push(MenuChoice {
                                text: text.clone(),
                                jump_key: "".to_string(),
                            });
                            current_index += 1;
                        }
                        ParseStatement::Jump { key } => {
                            choices.last_mut().unwrap().jump_key = key.clone();
                            current_index += 1;
                        }
                        ParseStatement::Dialogue { character_key, text }  => {
                            character_name = characters.get(character_key).unwrap().name.clone();
                            character_text = text.clone();

                            current_index += 1;
                        }
                        _ => {
                            break;
                        }
                    }
                }

                pages.push(Page {
                    index: current_index,
                    label: None,
                    content: Page_Content::Menu { character_name: character_name, text: character_text, choices: choices },
                    unconditional_jump: None,
                });
            }
            ParseStatement::Label { key } => {
                next_has_label = true;
                next_label = key.clone();

                current_index += 1;
            }
            ParseStatement::Jump { key } => {
                pages.last_mut().unwrap().unconditional_jump = Some(key.clone());
                current_index += 1;
            }
            ParseStatement::End {} => {
                pages.push(Page {
                    index: current_index,
                    label: None,
                    content: Page_Content::Dialogue {
                        character_name: "".to_string(),
                        text: "End".to_string(),
                    },
                    unconditional_jump: None,
                });
                current_index += 1;
            }
            _ => {
                current_index += 1;
            }
        }
        if current_index == logical_lines.len() - 1 {
            break;
        }
    }

    pages
}

fn find_label_index(logical_lines: Vec<ParseLogicalLine>, key: String) -> usize {
    for (index, line) in logical_lines.iter().enumerate() {
        match &line.statement {
            ParseStatement::Label { key: key_label } => {
                if key == *key_label {
                    return index;
                }
            }
            _ => (),
        }
    }
    return 0;
}

fn latex_output(pages: Vec<Page>) -> String {
    let mut output = String::new();

    output += "\\documentclass{beamer}\n\
    \\usepackage{hyperref}\n\
    \\title{Game Title}\n\
    \\author{Game Author}\n\
    \\date{\\today}\n\
    \\begin{document}\n\
    \\frame{\\titlepage}\n\
    ";

    for page in pages {
        output += "\\begin{frame}\n";
        let label_add =  if let Some(label) = page.label {
            format!("\\phantomsection\\hypertarget{{{}}}\n", label).to_string()
        } else {
            "".to_string()
        };
        match page.content {
            Page_Content::Dialogue {
                character_name,
                text,
            } => {
                output += format!(
                    "\\frametitle{{{}}}\n\
                    {}\
                    {}\n",
                    character_name, label_add, escape_for_latex(text)
                )
                .as_str();
            }
            Page_Content::Menu { character_name, text, choices } => {
                output += format!("\\frametitle{{{}}}\n", character_name).as_str();
                output += &label_add;
                output += format!("{}\n", escape_for_latex(text)).as_str();
                output += "\\begin{itemize}\n";
                for choice in choices {
                    output += format!("\\item \\hyperlink{{{}}}{{{}}}\n", choice.jump_key, escape_for_latex(choice.text)).as_str();
                }
                output += "\\end{itemize}\n";
            }
        }
        if let Some(jump) = page.unconditional_jump {
            output += format!("\\hyperlink{{{}}}{{\\beamergotobutton{{Next}}}}\n", jump).as_str();
        }
        output += "\\end{frame}\n";
    }

    output += "\\end{document}\n";

    output
}

fn escape_for_latex(text: String) -> String {
    if text.len() == 0 {
        return "~".to_string();
    }
    text.replace("$", "\\$").replace("%", "\\%").replace("#", "\\#").replace("_", "\\_")
}

fn clean_up_text(text: String) -> String {
    text.replace("\\\"", "\"").replace("\\n", "\n")
}

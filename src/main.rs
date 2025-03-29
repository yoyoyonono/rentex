use std::{collections::HashMap, fs};

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
    Show { key: String, location: Location },
    StageDirection { location: Location },
    Scene {},
}

#[derive(Debug, Clone)]
enum Location {
    Left,
    CenterLeft,
    Center,
    CenterRight,
    Right,
    Off,
}

#[derive(Debug, Clone)]
struct Character {
    name: String,
    color: String,
}

#[derive(Debug, Clone)]
struct Page {
    index: usize,
    label: Option<String>,
    text: PageText,
    images: [Option<String>; 5],
    unconditional_jump: Option<String>,
    end: bool,
}

#[derive(Debug, Clone)]
enum PageText {
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
    let script = std::fs::read_to_string("input/script.rpy").unwrap();
    let mut logical_lines: Vec<ParseLogicalLine> = Vec::<ParseLogicalLine>::new();
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
        if logical_lines.len() > 2 {
            let last_line = logical_lines.last().unwrap();
            let two_index = logical_lines.len() - 2;
            match &last_line.statement {
                ParseStatement::StageDirection { location } => {
                    let statement = logical_lines.get(two_index).unwrap().statement.clone();
                    match statement {
                        ParseStatement::Show { key, location: _ } => {
                            logical_lines.get_mut(two_index).unwrap().statement =
                                ParseStatement::Show {
                                    key: key,
                                    location: location.clone(),
                                };
                        }
                        _ => (),
                    }
                    logical_lines.pop();
                }
                _ => (),
            }
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
            ParseStatement::Show { key, location } => {
                println!("Show: {} at {:?}", key, location);
            }
            ParseStatement::StageDirection { location } => {
                println!("Stage Direction: {:?}", location);
            }
            ParseStatement::Scene {} => {
                println!("Scene");
            }
        }
    }

    let pages = traverse_game(logical_lines);

    println!("Pages: {:#?}", pages);

    let latex = latex_output(pages);

    std::fs::write("output/out.tex", latex).unwrap();
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
                statement: ParseStatement::Choice {
                    text: clean_up_text(text),
                },
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
    } else if line_trim.starts_with("show") {
        let line_new = line_trim.replace("show", "").trim().to_string();
        let line_split = line_new.split(" at ").collect::<Vec<&str>>();
        let key = line_split[0]
            .replace(":", "")
            .replace("flipped", "")
            .trim()
            .to_string();
        let location = if (line_split.len() > 1) {
            match line_split[1].trim() {
                "left" => Location::Left,
                "right" => Location::Right,
                _ => Location::Center,
            }
        } else {
            Location::Center
        };
        return Ok(ParseLogicalLine {
            indent: line.find("show").unwrap(),
            statement: ParseStatement::Show {
                key: key,
                location: location,
            },
        });
    } else if line_trim.starts_with("scene") {
        return Ok(ParseLogicalLine {
            indent: line.find("scene").unwrap(),
            statement: ParseStatement::Scene {},
        });
    } else if [
        "leftstage",
        "leftcenterstage",
        "centerstage",
        "rightcenterstage",
        "rightstage",
        "off_right",
        "off_left",
        "off_farright",
        "off_farleft",
        "percsuperleft",
        "percrightcenter",
        "xalign",
    ]
    .iter()
    .any(|x| line.contains(x))
    {
        let location = if line_trim.contains("off") {
            Location::Off
        } else if line_trim.contains("leftcenterstage") {
            Location::CenterLeft
        } else if line_trim.contains("rightcenterstage") {
            Location::CenterRight
        } else if line_trim.contains("leftstage") {
            Location::Left
        } else if line_trim.contains("rightstage") {
            Location::Right
        } else if line_trim.contains("xalign") {
            let xalign_index = line_trim.find("xalign").unwrap();
            let next_word = line_trim[xalign_index + 6..].split(" ").nth(1).unwrap();
            let xalign = next_word.parse::<f32>().unwrap();
            if xalign < 0.33 {
                Location::Left
            } else if xalign < 0.66 {
                Location::Center
            } else {
                Location::Right
            }
        } else {
            Location::Center
        };
        return Ok(ParseLogicalLine {
            indent: 8,
            statement: ParseStatement::StageDirection { location: location },
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
    let mut on_screen_characters = [None, None, None, None, None];

    loop {
        let line = &logical_lines[current_index];
        let statement = &line.statement;
        println!("Currently using: {:?}", statement);
        println!("Currently showing: {:?}", on_screen_characters);
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
                    text: PageText::Dialogue {
                        character_name: characters.get(character_key).unwrap().name.clone(),
                        text: text.clone(),
                    },
                    images: on_screen_characters.clone(),
                    unconditional_jump: None,
                    end: false,
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
                        ParseStatement::Dialogue {
                            character_key,
                            text,
                        } => {
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
                    text: PageText::Menu {
                        character_name: character_name,
                        text: character_text,
                        choices: choices,
                    },
                    images: [None, None, None, None, None],
                    unconditional_jump: None,
                    end: false,
                });
            }
            ParseStatement::Label { key } => {
                next_has_label = true;
                next_label = key.clone();

                current_index += 1;
            }
            ParseStatement::Jump { key } => {
                if pages.len() > 0 {
                    pages.last_mut().unwrap().unconditional_jump = Some(key.clone());
                }
                current_index += 1;
            }
            ParseStatement::End {} => {
                pages.push(Page {
                    index: current_index,
                    label: None,
                    text: PageText::Dialogue {
                        character_name: "".to_string(),
                        text: "End".to_string(),
                    },
                    images: on_screen_characters.clone(),
                    unconditional_jump: None,
                    end: true,
                });
                current_index += 1;
            }
            ParseStatement::Show { key, location } => {
                for i in 0..on_screen_characters.len() {
                    if let Some(character) = &on_screen_characters[i] {
                        if character.split(' ').nth(0).unwrap() == key.split(' ').nth(0).unwrap() {
                            on_screen_characters[i] = None;
                        }
                    }
                }
                match location {
                    Location::Left => {
                        on_screen_characters[0] = Some(key.clone());
                    }
                    Location::CenterLeft => {
                        on_screen_characters[1] = Some(key.clone());
                    }
                    Location::Center => {
                        on_screen_characters[2] = Some(key.clone());
                    }
                    Location::CenterRight => {
                        on_screen_characters[3] = Some(key.clone());
                    }
                    Location::Right => {
                        on_screen_characters[4] = Some(key.clone());
                    }
                    Location::Off => {}
                }
                current_index += 1;
            }
            ParseStatement::Scene {} => {
                on_screen_characters = [None, None, None, None, None];
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

    output += "\\documentclass[aspectratio=169]{beamer}\n\
    \\usepackage{hyperref}\n\
    \\beamertemplatenavigationsymbolsempty\n\
    \\title{Game Title}\n\
    \\author{Game Author}\n\
    \\date{\\today}\n\
    \\begin{document}\n\
    \\frame{\\titlepage}\n\
    ";

    for (index, page_iter) in pages.iter().enumerate() {
        let page = page_iter.clone();
        output += "\\begin{frame}\n";
        let label_add = if let Some(label) = page.label {
            format!("\\phantomsection\\hypertarget{{{}}}\n", label).to_string()
        } else {
            "".to_string()
        };
        let page_index_label = format!("\\phantomsection\\hypertarget{{pagenumber{}}}\n", index);
        if page.images != [None, None, None, None, None] {
            output += "\\begin{columns}\n";
            for image in page.images {
                if let Some(filename) = image {
                    if fs::exists(format!("output/images/{}.png", filename).as_str()).unwrap() {
                        output += "\\begin{column}{0.2\\textwidth}\n";
                        output += format!(
                            "\\includegraphics[width=\\textwidth]{{images/{}.png}}\n",
                            filename
                        )
                        .as_str();
                        output += "\\end{column}\n";
                    }
                } else {
                    output += "\\begin{column}{0.2\\textwidth}\n";
                    output += "\\end{column}\n";
                }
            }
            output += "\\end{columns}\n";
        }
        match page.text.clone() {
            PageText::Dialogue {
                character_name,
                text,
            } => {
                output += format!("\\frametitle{{{}}}\n", character_name).as_str();
                output += &label_add;
                output += &page_index_label;
                output += format!("{}\n", escape_for_latex(text)).as_str();
            }
            PageText::Menu {
                character_name,
                text,
                choices,
            } => {
                output += format!("\\frametitle{{{}}}\n", character_name).as_str();
                output += &label_add;
                output += &page_index_label;
                output += format!("{}\n", escape_for_latex(text)).as_str();
                output += "\\begin{itemize}\n";
                for choice in choices {
                    output += format!(
                        "\\item \\hyperlink{{{}}}{{{}}}\n",
                        choice.jump_key,
                        escape_for_latex(choice.text)
                    )
                    .as_str();
                }
                output += "\\end{itemize}\n";
            }
        }
        output += "\\vfill{}\n";
        output += "\\begin{flushright}\n";
        if let Some(jump) = page.unconditional_jump {
            output += format!("\\hyperlink{{{}}}{{\\beamergotobutton{{Next}}}}\n", jump).as_str();
        } else if !(matches!(page.text, PageText::Menu { .. }) || page.end) {
            output += format!(
                "\\hyperlink{{pagenumber{}}}{{\\beamergotobutton{{Next}}}}\n",
                index + 1
            )
            .as_str();
        }
        output += "\\end{flushright}\n";
        output += "\\end{frame}\n";
    }

    output += "\\end{document}\n";

    output
}

fn escape_for_latex(text: String) -> String {
    if text.len() == 0 {
        return "~".to_string();
    }
    text.replace("$", "\\$")
        .replace("%", "\\%")
        .replace("#", "\\#")
        .replace("_", "\\_")
}

fn clean_up_text(text: String) -> String {
    text.replace("\\\"", "\"").replace("\\n", "\n")
}

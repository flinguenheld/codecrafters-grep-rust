use std::env;
use std::io;
use std::process;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
enum Check {
    Ok,
    OkRepeat,
    EndRepeat,
    Optional,
    Nok,
}

fn pop_last_pattern(
    patterns: &mut Vec<Vec<Rc<dyn Fn(char) -> Check>>>,
) -> Option<Rc<dyn Fn(char) -> Check>> {
    let mut output = None;
    for p in patterns.iter_mut() {
        output = p.pop();
    }
    output
}

fn add_pattern(
    new_pattern: Rc<dyn Fn(char) -> Check>,
    patterns: &mut Vec<Vec<Rc<dyn Fn(char) -> Check>>>,
) {
    for p in patterns.iter_mut() {
        p.push(new_pattern.clone());
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let mut input_line = String::new();
    if io::stdin().read_line(&mut input_line).is_ok() {
        let mut patterns: Vec<Vec<Rc<dyn Fn(char) -> Check>>> = vec![Vec::new()];
        let mut temp_at_parenthesis: Vec<Vec<Rc<dyn Fn(char) -> Check>>> = vec![Vec::new()];
        let mut temps_at_pipes: Vec<Vec<Vec<Rc<dyn Fn(char) -> Check>>>> = Vec::new();

        let mut back_references: Vec<String> = vec![String::new()];
        let mut record_back_ref = false;

        if let Some(mut raw_pattern) = env::args().last() {
            let mut start_end = (false, false);
            if raw_pattern.starts_with('^') {
                raw_pattern.remove(0);
                start_end.0 = true;
            }
            if raw_pattern.ends_with('$') {
                raw_pattern.pop();
                start_end.1 = true;
            }

            let mut current = String::new();
            for c in raw_pattern.chars() {
                current.push(c);

                if current == "\\" {
                    continue;
                } else if current.starts_with('\\') {
                    match c {
                        'd' => {
                            add_pattern(
                                Rc::new(|c: char| {
                                    if c.is_ascii_digit() {
                                        Check::Ok
                                    } else {
                                        Check::Nok
                                    }
                                }),
                                &mut patterns,
                            );
                        }
                        'w' => {
                            add_pattern(
                                Rc::new(|c: char| {
                                    if c.is_ascii_alphanumeric() {
                                        Check::Ok
                                    } else {
                                        Check::Nok
                                    }
                                }),
                                &mut patterns,
                            );
                        }
                        _ if c.is_ascii_digit() && c != '0' => {
                            if let Some(index) = c.to_digit(10) {
                                if let Some(back) = back_references.get((index - 1) as usize) {
                                    for letter in back.chars() {
                                        add_pattern(
                                            Rc::new(move |ch: char| {
                                                if ch == letter {
                                                    Check::Ok
                                                } else {
                                                    Check::Nok
                                                }
                                            }),
                                            &mut patterns,
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    current.clear();
                } else if current.starts_with('[') && !current.ends_with(']') {
                    continue;
                } else if current.starts_with("[^") && current.ends_with(']') {
                    let characters: String = current
                        .chars()
                        .filter(|cc| cc.is_ascii_alphanumeric())
                        .collect();

                    let ascii_reverse: String = (1_u8..=126)
                        .map(|n| n as char)
                        .filter(|cc| !characters.contains(*cc))
                        .collect();

                    add_pattern(
                        Rc::new(move |ch: char| {
                            if ascii_reverse.contains(ch) {
                                Check::Ok
                            } else {
                                Check::Nok
                            }
                        }),
                        &mut patterns,
                    );
                    current.clear();
                } else if current.starts_with('[') && current.ends_with(']') {
                    let blah: String = current
                        .chars()
                        .filter(|cc| cc.is_ascii_alphanumeric())
                        .collect();
                    add_pattern(
                        Rc::new(move |ch: char| {
                            if blah.contains(ch) {
                                Check::Ok
                            } else {
                                Check::Nok
                            }
                        }),
                        &mut patterns,
                    );
                    current.clear();
                } else if current == "(" {
                    temp_at_parenthesis = patterns.clone();
                    record_back_ref = true;
                    current.clear();
                } else if current == "|" {
                    temps_at_pipes.push(patterns);
                    patterns = temp_at_parenthesis.clone();

                    record_back_ref = false;
                    if let Some(last) = back_references.last_mut() {
                        last.clear();
                    }
                    current.clear();
                } else if current == ")" {
                    for p in temps_at_pipes.iter_mut() {
                        patterns.append(p);
                    }
                    record_back_ref = false;
                    current.clear();
                } else if current == "+" {
                    println!("Add +");
                    if let Some(last_pat) = pop_last_pattern(&mut patterns) {
                        add_pattern(
                            Rc::new(move |ch: char| match (last_pat)(ch) {
                                Check::Ok => Check::OkRepeat,
                                _ => Check::EndRepeat,
                            }),
                            &mut patterns,
                        );
                    }
                    current.clear();
                } else if current == "?" {
                    println!("Add ?");
                    if let Some(last_pat) = pop_last_pattern(&mut patterns) {
                        add_pattern(
                            Rc::new(move |ch: char| match (last_pat)(ch) {
                                Check::Ok => Check::Ok,
                                _ => Check::Optional,
                            }),
                            &mut patterns,
                        );
                    }
                    current.clear();
                } else if current == "." {
                    println!("Add .");
                    add_pattern(Rc::new(move |_| Check::Ok), &mut patterns);
                    current.clear();
                } else {
                    println!("Add just a char: {}", c);
                    add_pattern(
                        Rc::new(move |ch: char| if ch == c { Check::Ok } else { Check::Nok }),
                        &mut patterns,
                    );
                    if record_back_ref {
                        if let Some(last) = back_references.last_mut() {
                            last.push_str(current.as_str());
                        }
                    }
                    current.clear();
                }
            }

            let found = match start_end {
                (true, false) => test_pattern(&input_line, &patterns, true),
                (false, true) => test_pattern(
                    &input_line.chars().rev().collect(),
                    &patterns
                        .iter()
                        .map(|p| p.iter().cloned().rev().collect())
                        .collect(),
                    true,
                ),
                (true, true) => {
                    test_pattern(&input_line, &patterns, true)
                        && test_pattern(
                            &input_line.chars().rev().collect(),
                            &patterns
                                .iter()
                                .map(|p| p.iter().cloned().rev().collect())
                                .collect(),
                            true,
                        )
                }
                _ => test_pattern(&input_line, &patterns, false),
            };

            match found {
                true => {
                    println!("Found");
                    process::exit(0)
                }
                false => {
                    println!("Not found");
                    process::exit(1)
                }
            }
        }
    }

    process::exit(1)
}

fn test_pattern(
    input_line: &String,
    patterns: &Vec<Vec<Rc<dyn Fn(char) -> Check>>>,
    on_start_only: bool,
) -> bool {
    for pattern in patterns {
        'aaa: for i in 0..input_line.chars().count() {
            let mut pat_iter = pattern.iter();
            let mut inp_iter = input_line.chars().skip(i).peekable();

            let mut ok_repeat_validation = false;

            'bbb: loop {
                if let Some(p) = pat_iter.next() {
                    'ccc: while let Some(c) = inp_iter.peek() {
                        println!("Testing this char: {}", c);
                        match (p)(*c) {
                            Check::Ok => {
                                dbg!("Ok");
                                inp_iter.next();
                                continue 'bbb;
                            }
                            Check::OkRepeat => {
                                dbg!("Ok repeat");
                                inp_iter.next();
                                ok_repeat_validation = true;
                                continue 'ccc;
                            }
                            Check::EndRepeat => {
                                dbg!("End repeat");
                                if ok_repeat_validation {
                                    ok_repeat_validation = false;

                                    continue 'bbb;
                                } else {
                                    continue 'aaa;
                                }
                            }
                            Check::Optional => {
                                dbg!("Optional");
                                // inp_iter.next();
                                continue 'bbb;
                            }
                            Check::Nok => {
                                dbg!("Nok");
                                if on_start_only {
                                    return false;
                                } else {
                                    continue 'aaa;
                                }
                            }
                        }
                    }

                    if pat_iter.cloned().next().is_none() {
                        // Special check for + is in the last position
                        if ok_repeat_validation {
                            dbg!("Validate last +");
                            return true;
                        }

                        // Special check for ? is in the last position
                        if (p)('\0') == Check::Optional {
                            dbg!("Validate last optional");
                            return true;
                        }
                    }

                    continue 'aaa;
                } else {
                    return true;
                }
            }
        }
    }

    false
}

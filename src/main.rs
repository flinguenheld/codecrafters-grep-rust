use std::env;
use std::io;
use std::process;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
enum Pouet {
    Ok,
    OkRepeat,
    EndRepeat,
    Optional,
    Nok,
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let mut input_line = String::new();
    if io::stdin().read_line(&mut input_line).is_ok() {
        let mut pat: Vec<Rc<dyn Fn(char) -> Pouet>> = Vec::new();
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
                            pat.push(Rc::new(|c: char| {
                                if c.is_ascii_digit() {
                                    Pouet::Ok
                                } else {
                                    Pouet::Nok
                                }
                            }));
                        }
                        'w' => {
                            pat.push(Rc::new(|c: char| {
                                if c.is_ascii_alphanumeric() {
                                    Pouet::Ok
                                } else {
                                    Pouet::Nok
                                }
                            }));
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

                    // Farfetch -_-'

                    pat.push(Rc::new(move |ch: char| {
                        if ascii_reverse.contains(ch) {
                            Pouet::Ok
                        } else {
                            Pouet::Nok
                        }
                    }));
                    current.clear();
                } else if current.starts_with('[') && current.ends_with(']') {
                    let blah: String = current
                        .chars()
                        .filter(|cc| cc.is_ascii_alphanumeric())
                        .collect();
                    pat.push(Rc::new(move |ch: char| {
                        if blah.contains(ch) {
                            Pouet::Ok
                        } else {
                            Pouet::Nok
                        }
                    }));
                    current.clear();
                } else if current == "+" {
                    println!("Add +");
                    if let Some(last_pat) = pat.last().cloned() {
                        pat.push(Rc::new(move |ch: char| match (last_pat)(ch) {
                            Pouet::Ok => Pouet::OkRepeat,
                            _ => Pouet::EndRepeat,
                        }));
                    }
                    current.clear();
                } else if current == "?" {
                    println!("Add ?");
                    if let Some(last_pat) = pat.pop() {
                        pat.push(Rc::new(move |ch: char| match (last_pat)(ch) {
                            Pouet::Ok => Pouet::Ok,
                            _ => Pouet::Optional,
                        }));
                    }
                    current.clear();
                } else {
                    println!("Add just a char: {}", c);
                    pat.push(Rc::new(
                        move |ch: char| if ch == c { Pouet::Ok } else { Pouet::Nok },
                    ));
                    current.clear();
                }
            }

            let found = match start_end {
                (true, false) => test_pattern(&input_line, &pat, true),
                (false, true) => test_pattern(
                    &input_line.chars().rev().collect(),
                    &pat.iter().rev().cloned().collect(),
                    true,
                ),
                (true, true) => {
                    test_pattern(&input_line, &pat, true)
                        && test_pattern(
                            &input_line.chars().rev().collect(),
                            &pat.iter().rev().cloned().collect(),
                            true,
                        )
                }
                _ => test_pattern(&input_line, &pat, false),
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
    pattern: &Vec<Rc<dyn Fn(char) -> Pouet>>,
    on_start_only: bool,
) -> bool {
    'aaa: for i in 0..input_line.chars().count() {
        let mut pat_iter = pattern.iter();
        let mut inp_iter = input_line.chars().skip(i).peekable();

        'bbb: loop {
            if let Some(p) = pat_iter.next() {
                'ccc: while let Some(c) = inp_iter.peek() {
                    println!("Testing this char: {}", c);
                    match (p)(*c) {
                        Pouet::Ok => {
                            dbg!("Ok");
                            inp_iter.next();
                            continue 'bbb;
                        }
                        Pouet::OkRepeat => {
                            dbg!("Ok repeat");
                            inp_iter.next();
                            continue 'ccc;
                        }
                        Pouet::EndRepeat => {
                            dbg!("End repeat");
                            continue 'bbb;
                        }
                        Pouet::Optional => {
                            dbg!("Optional");
                            // inp_iter.next();
                            // AdAPT
                            continue 'bbb;
                        }
                        Pouet::Nok => {
                            dbg!("Nok");
                            if on_start_only {
                                return false;
                            } else {
                                continue 'aaa;
                            }
                        }
                    }
                }

                // Check ?
                if (p)('\0') == Pouet::Optional && pat_iter.cloned().next().is_none() {
                    dbg!("Validate last optional");
                    return true;
                }

                continue 'aaa;
            } else {
                return true;
            }
        }
    }

    false
}

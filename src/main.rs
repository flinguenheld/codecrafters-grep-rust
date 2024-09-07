use std::env;
use std::io;
use std::process;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let mut input_line = String::new();
    if io::stdin().read_line(&mut input_line).is_ok() {
        let mut pat: Vec<Box<dyn Fn(char) -> bool>> = Vec::new();
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
                            pat.push(Box::new(|c: char| c.is_ascii_digit()));
                        }
                        'w' => {
                            pat.push(Box::new(|c: char| c.is_ascii_alphanumeric()));
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

                    pat.push(Box::new(move |ch: char| ascii_reverse.contains(ch)));
                } else if current.starts_with('[') && current.ends_with(']') {
                    let blah: String = current
                        .chars()
                        .filter(|cc| cc.is_ascii_alphanumeric())
                        .collect();
                    pat.push(Box::new(move |ch: char| blah.contains(ch)));
                } else {
                    println!("Add just a char: {}", c);
                    pat.push(Box::new(move |ch: char| ch == c));
                    current.clear();
                }
            }

            let found = match start_end {
                (true, false) => test_pattern(&input_line, &pat, true),
                (false, true) => test_pattern_reverse(&input_line, &pat, true),
                (true, true) => {
                    test_pattern(&input_line, &pat, true)
                        && test_pattern_reverse(&input_line, &pat, true)
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
    pattern: &Vec<Box<dyn Fn(char) -> bool>>,
    on_start_only: bool,
) -> bool {
    'aaa: for i in 0..input_line.chars().count() {
        let mut pat_iter = pattern.iter();
        let mut inp_iter = input_line.chars().skip(i);

        loop {
            if let Some(p) = pat_iter.next() {
                if let Some(c) = inp_iter.next() {
                    if !(p)(c) {
                        if on_start_only {
                            return false;
                        } else {
                            continue 'aaa;
                        }
                    }
                } else {
                    continue 'aaa;
                }
            } else {
                return true;
            }
        }
    }

    false
}
fn test_pattern_reverse(
    input_line: &String,
    pattern: &Vec<Box<dyn Fn(char) -> bool>>,
    // Remove that ?
    on_end_only: bool,
) -> bool {
    'aaa: for i in 0..input_line.chars().count() {
        let mut pat_iter = pattern.iter().rev();
        let mut inp_iter = input_line.chars().rev().skip(i);

        loop {
            if let Some(p) = pat_iter.next() {
                if let Some(c) = inp_iter.next() {
                    if !(p)(c) {
                        if on_end_only {
                            return false;
                        } else {
                            continue 'aaa;
                        }
                    }
                } else {
                    continue 'aaa;
                }
            } else {
                return true;
            }
        }
    }

    false
}

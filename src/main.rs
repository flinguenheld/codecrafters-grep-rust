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
        if let Some(raw_pattern) = env::args().last() {
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
                    let blah: String = current
                        .chars()
                        .filter(|cc| cc.is_ascii_alphanumeric())
                        .collect();

                    let wooooo: String = (1_u8..=126)
                        .map(|i| i as char)
                        .filter(|cc| !blah.contains(*cc))
                        .collect();

                    // Farfetch -_-'

                    // dbg!(&wooooo);

                    pat.push(Box::new(move |ch: char| wooooo.contains(ch)));
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

            let mut found = false;

            'aaa: for i in 0..input_line.chars().count() {
                let mut pat_iter = pat.iter();
                let mut inp_iter = input_line.chars().skip(i).into_iter();

                loop {
                    if let Some(p) = pat_iter.next() {
                        if let Some(c) = inp_iter.next() {
                            if !(p)(c) {
                                continue 'aaa;
                            }
                        } else {
                            continue 'aaa;
                        }
                    } else {
                        found = true;
                        break;
                    }
                }

                // for (c, p) in input_line.chars().skip(i).zip(pat.iter()) {
                //     if !(p)(c) {
                //         continue 'aaa;
                //     }
                // }
                // println!("Whole pattern checked");
                // found = true;
            }

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

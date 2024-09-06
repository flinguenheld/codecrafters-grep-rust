use std::env;
use std::io;
use std::process;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let mut input_line = String::new();
    if io::stdin().read_line(&mut input_line).is_ok() {
        let mut found = false;
        // if let Some(pattern) = env::args().last() {
        for arg in env::args() {
            match arg.as_str() {
                a if arg.starts_with('[') && arg.ends_with(']') => {
                    let letters = a
                        .chars()
                        .filter(|c| c.is_ascii_alphabetic())
                        .collect::<String>();

                    if input_line.chars().any(|c| letters.contains(c)) {
                        found = true;
                        break;
                    }
                }

                "\\d" => {
                    if input_line.chars().any(|c| c.is_ascii_digit()) {
                        found = true;
                        break;
                    }
                }
                "\\w" => {
                    if input_line
                        .chars()
                        .any(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        found = true;
                        break;
                    }
                }
                _ => {
                    if input_line.contains(arg.as_str()) {
                        found = true;
                        break;
                    }
                }
            }
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

        // if env::args().nth(1).unwrap() != "-E" {
        //     println!("Expected first argument to be '-E'");
        //     process::exit(1);
        // }

        // let pattern = env::args().nth(2).unwrap();
        // let mut input_line = String::new();

        // Uncomment this block to pass the first stage
        // if match_pattern(&input_line, &pattern) {
        //     println!("Pattern '{}' found", pattern);
        //     process::exit(0)
        // } else {
        //     process::exit(1)
        // }
    }
    process::exit(1)
}

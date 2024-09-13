use std::env;
use std::io;
use std::process;
use std::rc::Rc;

#[derive(PartialEq, Eq, Debug)]
enum Check {
    Ok,
    OkRepeat,
    EndRepeat,
    Optional,
    BackRefRecordStart,
    BackRefRecordEnd,
    BackRefCall(usize),
    BackRefValidated,
    End,
    Nok,
}

type Pattern = Rc<dyn Fn(char) -> Check>;

fn debug(txt: &str, print: bool) {
    if print {
        println!("Debug: {}", txt);
    }
}

/// Each pipe creates two new patternS which are added to the current list.
/// So this func add the 'new_pattern' (for one char) at the end of all patternS.
fn add_pattern(new_pattern: Pattern, patterns: &mut [Vec<Pattern>]) {
    for p in patterns.iter_mut() {
        p.push(new_pattern.clone());
    }
}
fn pop_last_pattern(patterns: &mut [Vec<Pattern>]) -> Option<Pattern> {
    let mut output = None;
    for p in patterns.iter_mut() {
        output = p.pop();
    }
    output
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    let db = env::args().any(|a| a == "-d");

    let mut input_line = String::new();
    if io::stdin().read_line(&mut input_line).is_ok() {
        let mut patterns: Vec<Vec<Pattern>> = vec![Vec::new()];
        let mut temp_at_parenthesis: Vec<Vec<Pattern>> = vec![Vec::new()];
        let mut temps_at_pipes: Vec<Vec<Vec<Pattern>>> = Vec::new();

        if let Some(raw_pattern) = env::args().last() {
            let on_start = raw_pattern.starts_with('^');

            let mut current = String::new();
            for c in raw_pattern.chars() {
                current.push(c);

                if current == "^" && on_start {
                    current.clear();
                    continue;
                } else if current == "\\" {
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
                                debug(&format!("Add call to back reference {}", index - 1), db);
                                add_pattern(
                                    Rc::new(move |_| Check::BackRefCall((index - 1) as usize)),
                                    &mut patterns,
                                );
                            }
                        }
                        _ => {}
                    }
                    current.clear();
                } else if current == "$" {
                    add_pattern(Rc::new(move |_| Check::End), &mut patterns);
                    current.clear();
                } else if current.starts_with('[') && !current.ends_with(']') {
                    continue;
                } else if current.starts_with("[^") && current.ends_with(']') {
                    dbg!("Add negative");
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
                    add_pattern(Rc::new(|_| Check::BackRefRecordStart), &mut patterns);

                    current.clear();
                } else if current == "|" {
                    temps_at_pipes.push(patterns);
                    patterns = temp_at_parenthesis.clone();
                    add_pattern(Rc::new(|_| Check::BackRefRecordStart), &mut patterns);

                    current.clear();
                } else if current == ")" {
                    for p in temps_at_pipes.iter_mut() {
                        patterns.append(p);
                    }

                    add_pattern(Rc::new(|_| Check::BackRefRecordEnd), &mut patterns);
                    current.clear();
                } else if current == "+" {
                    if let Some(last_pat) = pop_last_pattern(&mut patterns) {
                        debug("Add +", db);
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
                    if let Some(last_pat) = pop_last_pattern(&mut patterns) {
                        debug("Add ?", db);
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
                    debug("Add .", db);
                    add_pattern(Rc::new(move |_| Check::Ok), &mut patterns);
                    current.clear();
                } else {
                    debug(&format!("Add a simple char: '{}'", c), db);
                    add_pattern(
                        Rc::new(move |ch: char| if ch == c { Check::Ok } else { Check::Nok }),
                        &mut patterns,
                    );
                    current.clear();
                }
            }

            debug("----- TESTS -----", db);
            match test_pattern(&input_line, &patterns, on_start, db) {
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

    println!("Error");
    process::exit(1)
}

fn test_pattern(
    input_line: &str,
    patterns: &[Vec<Pattern>],
    on_start_only: bool,
    vb: bool,
) -> bool {
    for (nb, patt) in patterns.iter().rev().enumerate() {
        debug(&format!("Pattern {} on {}", nb, patterns.len()), vb);
        'aaa: for i in 0..input_line.chars().count() {
            let mut pat_iter = patt.iter();
            let mut inp_iter = input_line.chars().skip(i).peekable();

            let mut ok_repeat_validation = false;

            // (pattern, validated = true)
            let mut back_references: Vec<(Vec<Pattern>, bool)> = Vec::new();
            let mut back_ref_last_letter_recorded = '\0';
            let mut back_ref_pattern_index = 0; // Used to read a backref

            'bbb: loop {
                if let Some(p) = pat_iter.next() {
                    'ccc: while let Some(c) = inp_iter.peek() {
                        debug(&format!("Test -> '{}'", c), vb);
                        match (p)(*c) {
                            Check::End => {
                                continue 'aaa;
                            }
                            Check::Ok => {
                                debug("Ok", vb);
                                back_ref_last_letter_recorded =
                                    add_back_ref_pattern_to_all(*c, &mut back_references);

                                inp_iter.next();
                                continue 'bbb;
                            }
                            Check::OkRepeat => {
                                debug("Ok repeat", vb);
                                back_ref_last_letter_recorded =
                                    add_back_ref_pattern_to_all(*c, &mut back_references);

                                inp_iter.next();
                                ok_repeat_validation = true;
                                continue 'ccc;
                            }
                            Check::EndRepeat => {
                                debug("End repeat", vb);
                                if ok_repeat_validation {
                                    ok_repeat_validation = false;

                                    continue 'bbb;
                                } else {
                                    continue 'aaa;
                                }
                            }
                            Check::Optional => {
                                debug("Optional", vb);
                                continue 'bbb;
                            }
                            Check::Nok => {
                                debug("Nok", vb);
                                if on_start_only {
                                    return false;
                                } else {
                                    continue 'aaa;
                                }
                            }
                            Check::BackRefRecordStart => {
                                debug("Back ref record -> start", vb);
                                back_references.push((Vec::new(), false));

                                continue 'bbb;
                            }
                            Check::BackRefRecordEnd => {
                                debug("Back ref record -> end", vb);

                                if let Some(back_ref) =
                                    back_references.iter_mut().filter(|br| !br.1).next_back()
                                {
                                    // Replace the last one check's return
                                    back_ref.0.pop();

                                    let letter = back_ref_last_letter_recorded;
                                    back_ref.0.push(Rc::new(move |ch: char| {
                                        if ch == letter {
                                            Check::BackRefValidated
                                        } else {
                                            Check::Nok
                                        }
                                    }));
                                    back_ref.1 = true;
                                }

                                continue 'bbb;
                            }
                            Check::BackRefCall(n) => {
                                debug(&format!("Call back ref {} with: '{}'", n, c), vb);
                                if let Some(back_ref) = back_references.get(n) {
                                    if let Some(back_ref_test) =
                                        back_ref.0.get(back_ref_pattern_index)
                                    {
                                        match (back_ref_test)(*c) {
                                            Check::Ok => {
                                                back_ref_last_letter_recorded =
                                                    add_back_ref_pattern_to_all(
                                                        *c,
                                                        &mut back_references,
                                                    );

                                                debug(&format!("Back ref Ok with: '{}'", c), vb);
                                                back_ref_pattern_index += 1;
                                                inp_iter.next();
                                                continue 'ccc;
                                            }
                                            Check::BackRefValidated => {
                                                back_ref_last_letter_recorded =
                                                    add_back_ref_pattern_to_all(
                                                        *c,
                                                        &mut back_references,
                                                    );

                                                debug("Back ref Validated", vb);
                                                inp_iter.next();
                                                back_ref_pattern_index = 0;
                                                continue 'bbb;
                                            }
                                            _ => {
                                                debug(&format!("Back ref Nok with: '{}'", c), vb);
                                                continue 'aaa;
                                            }
                                        }
                                    }
                                } else {
                                    println!("Error unreachable back reference");
                                    process::exit(1)
                                }
                            }
                            _ => {}
                        }
                    }

                    if (p)('\0') == Check::End {
                        return true;
                    }

                    if pat_iter.next().cloned().is_none() {
                        let result = (p)('\0');
                        debug(&format!("Last pattern, result -> {:?}", result), vb);

                        // Validation if the last char is a special pattern
                        if result != Check::Nok
                            && (ok_repeat_validation || result != Check::EndRepeat)
                        {
                            debug("Special last pattern validated", vb);
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

/// Add the letter to all backref which are not already validated.
fn add_back_ref_pattern_to_all(letter: char, back_references: &mut [(Vec<Pattern>, bool)]) -> char {
    for back_ref in back_references.iter_mut().filter(|br| !br.1) {
        back_ref.0.push(Rc::new(
            move |ch: char| {
                if ch == letter {
                    Check::Ok
                } else {
                    Check::Nok
                }
            },
        ));
    }

    letter
}

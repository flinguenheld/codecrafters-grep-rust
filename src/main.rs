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

// echo -n "3 red squares and 3 red circles" | ./your_program.sh  "(\d+) (\w+) squares and \1 \2 circles"
// echo -n "howwdy heeey there, howwdy heeey" | ./your_program.sh  "(how+dy) (he?y) there, \1 \2"
// echo -n "cat and fish, cat with fish" | ./your_program.sh  "(c.t|d.g) and (f..h|b..d), \1 with \2"

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
    println!("Add char to {} pattern", patterns.len());
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

        if let Some(mut raw_pattern) = env::args().last() {
            let mut start_end = (false, false);
            if raw_pattern.starts_with('^') {
                raw_pattern.remove(0);
                start_end.0 = true;
            }
            // if raw_pattern.ends_with('$') {
            //     raw_pattern.pop();
            //     start_end.1 = true;
            // }

            let mut current = String::new();
            for c in raw_pattern.chars() {
                current.push(c);

                dbg!(&current);

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
                            dbg!("Add backref");
                            if let Some(index) = c.to_digit(10) {
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
                    current.clear();
                }
            }

            let found = match start_end {
                (true, _) => test_pattern(&input_line, &patterns, true),
                // (false, true) => test_pattern(
                //     &input_line.chars().rev().collect(),
                //     &patterns
                //         .iter()
                //         .map(|p| p.iter().cloned().rev().collect())
                //         .collect(),
                //     true,
                //     &mut back_ref_new_generation,
                // ),
                // (true, true) => {
                //     test_pattern(&input_line, &patterns, true, &mut back_ref_new_generation)
                //         && test_pattern(
                //             &input_line.chars().rev().collect(),
                //             &patterns
                //                 .iter()
                //                 .map(|p| p.iter().cloned().rev().collect())
                //                 .collect(),
                //             true,
                //             &mut back_ref_new_generation,
                //         )
                // }
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
    // back_ref_new_generation: &mut Vec<Vec<Rc<dyn Fn(char) -> Check>>>,
) -> bool {
    println!("nB of patterns : {}", patterns.len());

    for pattern in patterns {
        'aaa: for i in 0..input_line.chars().count() {
            println!("new start at {}", i);
            let mut pat_iter = pattern.iter();
            let mut inp_iter = input_line.chars().skip(i).peekable();

            let mut ok_repeat_validation = false;
            let mut back_ref_record = false;
            let mut back_ref_current = String::new();
            let mut back_ref_index = 0;

            let mut back_ref_new_generation: Vec<Vec<Rc<dyn Fn(char) -> Check>>> = Vec::new();

            'bbb: loop {
                if let Some(p) = pat_iter.next() {
                    'ccc: while let Some(c) = inp_iter.peek() {
                        println!("Testing this char: {}", c);
                        match (p)(*c) {
                            Check::End => {
                                continue 'aaa;
                                // println!("end ???");

                                // if inp_iter.next() == None {
                                //     println!("bah");
                                //     return true;
                                // } else {
                                //     // inp_iter.next();
                                //     continue 'bbb;
                                // }
                            }
                            Check::Ok => {
                                dbg!("Ok");

                                // Put that for all check ?
                                if back_ref_record {
                                    back_ref_current.push(*c);
                                }

                                inp_iter.next();
                                continue 'bbb;
                            }
                            Check::OkRepeat => {
                                dbg!("Ok repeat");
                                // Put that for all check ?
                                if back_ref_record {
                                    back_ref_current.push(*c);
                                }
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

                                // TODO: OPTIONAL ??????
                                // back_ref_current.clear();

                                if on_start_only {
                                    return false;
                                } else {
                                    continue 'aaa;
                                }
                            }
                            Check::BackRefRecordStart => {
                                dbg!("Back Ref record Start");
                                back_ref_record = true;
                                back_ref_current.clear();
                                continue 'bbb;
                            }
                            Check::BackRefRecordEnd => {
                                back_ref_record = false;

                                dbg!("Back Ref record END");

                                // TODO: With a map + collect ?
                                if !back_ref_current.is_empty() {
                                    println!("---------> Add this back ref: {}", back_ref_current);
                                    let mut aaa: Vec<Rc<dyn Fn(char) -> Check>> = Vec::new();
                                    for (i, c) in back_ref_current.char_indices() {
                                        if i < back_ref_current.chars().count() - 1 {
                                            aaa.push(Rc::new(move |ch: char| {
                                                if ch == c {
                                                    Check::Ok
                                                } else {
                                                    Check::Nok
                                                }
                                            }));
                                        } else {
                                            aaa.push(Rc::new(move |ch: char| {
                                                if ch == c {
                                                    Check::BackRefValidated
                                                } else {
                                                    Check::Nok
                                                }
                                            }));
                                        }
                                    }
                                    // aaa.push(Rc::new(|_| Check::BackRefValidated));
                                    back_ref_new_generation.push(aaa);
                                }

                                continue 'bbb;
                            }
                            Check::BackRefCall(n) => {
                                println!("Back ref length: {}", back_ref_new_generation.len());
                                println!("Back ref {} in progress with : {}", n, c);
                                if let Some(back_ref) = back_ref_new_generation.get(n) {
                                    if let Some(back_ref_test) = back_ref.get(back_ref_index) {
                                        dbg!((back_ref_test)(*c));
                                        match (back_ref_test)(*c) {
                                            Check::Ok => {
                                                println!("Back ref ok with the letter: {}", c);
                                                back_ref_index += 1;
                                                inp_iter.next();
                                                continue 'ccc;
                                            }
                                            Check::BackRefValidated => {
                                                println!("Back ref validated -> go the next pattern char");
                                                inp_iter.next();
                                                back_ref_index = 0;
                                                continue 'bbb;
                                            }
                                            _ => {
                                                println!("Back ref fail with the letter: {}", c);
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
                        println!("hhhaaaa end");
                        return true;
                    }

                    println!("Check if is last ???");

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

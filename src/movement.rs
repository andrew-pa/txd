// movements:
// hjkl: ±1 char/line
// w: forward one word
// b: backward one word
// e: forward one word, place at end
// f[char]/F[char]: scan forward/backward for char, place cursor on it
// t[char]/T[char]: scan forward/backward for char, place cursor before/after it
// $: end of line
// ^: start of line
// <number>[mov]: repeated movement n times
#[derive(Debug, Clone)]
pub enum Movement {
    Char(bool),
    Line(bool),
    Word(bool, bool), // (forward, place at end)
    CharScan {
        query: char, forwards: bool, place_besides: bool
    },
    StartOfLine,
    WholeLine,
    EndOfLine,
    Rep(usize, Box<Movement>)
}

impl Movement {
    pub fn parse(s: &str, first: bool) -> Option<Movement> {
        //println!("parse movment {}", s);
        use self::Movement::*;
        let mut cs = s.char_indices();
        match cs.next() {
            Some((i, c)) => {
                if c.is_digit(10) {
                    let start = i; let mut end = 0;
                    for (j,c) in cs {
                        if !c.is_digit(10) { end = j; break; }
                    }
                    let sp = s.split_at(end);
                    sp.0.parse::<usize>().ok().and_then(|n| Movement::parse(sp.1,false).map(|m| Rep(n,Box::new(m)))) 
                } else {
                    match c {
                        'h' => Some(Char(false)),
                        'j' => Some(Line(false)),
                        'k' => Some(Line(true)),
                        'l' => Some(Char(true)),
                        'w' => Some(Word(true, false)),
                        'b' => Some(Word(false, false)),
                        'e' => Some(Word(false, true)),
                        '^' => Some(StartOfLine),
                        'J' => Some(WholeLine),
                        '$' => Some(EndOfLine),
                        't' => cs.next().map(|(_,q)| CharScan { query: q, forwards: true, place_besides: true }),
                        'T' => cs.next().map(|(_,q)| CharScan { query: q, forwards: false, place_besides: true }),
                        'f' => cs.next().map(|(_,q)| CharScan { query: q, forwards: true, place_besides: false }),
                        'F' => cs.next().map(|(_,q)| CharScan { query: q, forwards: false, place_besides: false }),
                        _ => if !first {
                            match c {
                                'd' => Some(WholeLine),
                                'c' => Some(WholeLine),
                                _ => None
                            }
                        } else { None }
                    }
                }
            },
            None => None
        }
    }
}



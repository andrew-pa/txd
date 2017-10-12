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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Inclusion {
    Exclusive,
    Inclusive,
    Linewise
}

#[derive(Debug, Clone)]
pub enum Movement {
    Char(bool /*left/right*/),
    Line(bool /*up/down*/, Inclusion),
    Word(bool /*forwards/backwards*/, Inclusion),
    CharScan {
        query: char, direction: bool, inclusion: Inclusion, place_to_side: bool
    },
    StartOfLine,
    EndOfLine,
    Rep(usize, Box<Movement>)
}

impl Movement {
    pub fn inclusion_mode(&self) -> Inclusion {
        match self {
            &Movement::Char(_) => Inclusion::Exclusive,
            &Movement::Line(_, i) => i,
            &Movement::Word(_, i) => i,
            &Movement::CharScan { inclusion: i, .. } => i,
            &Movement::StartOfLine => Inclusion::Exclusive,
            &Movement::EndOfLine => Inclusion::Inclusive,
            &Movement::Rep(_, ref mv) => mv.inclusion_mode()
        }
    }
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
                        'j' => Some(Line(false, Inclusion::Linewise)),
                        'k' => Some(Line(true, Inclusion::Linewise)),
                        'l' => Some(Char(true)),
                        'w' => Some(Word(true, Inclusion::Exclusive)),
                        'b' => Some(Word(false, Inclusion::Exclusive)),
                        'e' => Some(Word(false, Inclusion::Inclusive)),
                        '^' => Some(StartOfLine),
                        'J' => Some(Line(false, Inclusion::Inclusive)),
                        '$' => Some(EndOfLine),
                        't' => cs.next().map(|(_,q)| CharScan { query: q, inclusion: Inclusion::Inclusive, direction: true, place_to_side: true }),
                        'T' => cs.next().map(|(_,q)| CharScan { query: q, inclusion: Inclusion::Exclusive, direction: false, place_to_side: true }),
                        'f' => cs.next().map(|(_,q)| CharScan { query: q, inclusion: Inclusion::Inclusive, direction: true, place_to_side: false }),
                        'F' => cs.next().map(|(_,q)| CharScan { query: q, inclusion: Inclusion::Exclusive, direction: false, place_to_side: false }),
                        _ => if !first {
                            match c {
                                'd' => Some(Line(false, Inclusion::Inclusive)),
                                'c' => Some(Line(false, Inclusion::Inclusive)),
                                'y' => Some(Line(false, Inclusion::Inclusive)),
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



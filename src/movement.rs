// movements:
// hjkl: ±1 char/line
// w: forward one word
// b: backward one word
// e: forward one word, place at end
// <number>[mov]: repeated movement n times
#[derive(Debug, Clone)]
pub enum Movement {
    Char(bool),
    Line(bool),
    Word(bool, bool),
    Rep(usize, Box<Movement>)
}

impl Movement {
    pub fn parse(s: &str) -> Option<Movement> {
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
                    sp.0.parse::<usize>().ok().and_then(|n| Movement::parse(sp.1).map(|m| Rep(n,Box::new(m)))) 
                } else {
                    match c {
                        'h' => Some(Char(false)),
                        'j' => Some(Line(false)),
                        'k' => Some(Line(true)),
                        'l' => Some(Char(true)),
                        'w' => Some(Word(false, false)),
                        'b' => Some(Word(true, false)),
                        'e' => Some(Word(false, true)),
                        _ => None
                    }
                }
            },
            None => None
        }
    }
}



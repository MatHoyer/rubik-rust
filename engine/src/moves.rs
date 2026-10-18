//! Face turns in standard notation, and parsing a mix.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    U,
    R,
    F,
    D,
    L,
    B,
}

impl Face {
    pub const ALL: [Face; 6] = [Face::U, Face::R, Face::F, Face::D, Face::L, Face::B];

    fn from_char(c: char) -> Option<Face> {
        Some(match c {
            'U' => Face::U,
            'R' => Face::R,
            'F' => Face::F,
            'D' => Face::D,
            'L' => Face::L,
            'B' => Face::B,
            _ => return None,
        })
    }
}

/// A face turned clockwise `turns` quarter turns (1, 2 or 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub face: Face,
    pub turns: u8,
}

impl Move {
    pub fn inverse(self) -> Move {
        Move {
            face: self.face,
            turns: 4 - self.turns,
        }
    }
}

pub const ALL_MOVES: [Move; 18] = {
    let mut all = [Move {
        face: Face::U,
        turns: 1,
    }; 18];
    let mut i = 0;
    while i < 18 {
        all[i] = Move {
            face: Face::ALL[i / 3],
            turns: (i % 3) as u8 + 1,
        };
        i += 1;
    }
    all
};

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let suffix = match self.turns {
            1 => "",
            2 => "2",
            _ => "'",
        };
        write!(f, "{:?}{suffix}", self.face)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    InvalidMove(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "empty mix"),
            ParseError::InvalidMove(m) => write!(
                f,
                "invalid move '{m}' (expected one of F R U B L D, optionally followed by ' or 2)"
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a whitespace-separated mix such as `"F R U2 B' L' D'"`.
pub fn parse(mix: &str) -> Result<Vec<Move>, ParseError> {
    let moves = mix
        .split_whitespace()
        .map(parse_move)
        .collect::<Result<Vec<_>, _>>()?;
    if moves.is_empty() {
        return Err(ParseError::Empty);
    }
    Ok(moves)
}

fn parse_move(token: &str) -> Result<Move, ParseError> {
    let invalid = || ParseError::InvalidMove(token.to_string());
    let mut chars = token.chars();
    let face = chars.next().and_then(Face::from_char).ok_or_else(invalid)?;
    let turns = match chars.as_str() {
        "" => 1,
        "2" => 2,
        "'" => 3,
        _ => return Err(invalid()),
    };
    Ok(Move { face, turns })
}

/// The sequence that undoes `moves`.
pub fn inverse(moves: &[Move]) -> Vec<Move> {
    moves.iter().rev().map(|m| m.inverse()).collect()
}

pub fn format(moves: &[Move]) -> String {
    moves
        .iter()
        .map(Move::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_valid_mix() {
        let moves = parse("F R U2 B' L' D'").unwrap();
        assert_eq!(format(&moves), "F R U2 B' L' D'");
    }

    #[test]
    fn accepts_any_whitespace() {
        let moves = parse("  F\tR   U2 \n").unwrap();
        assert_eq!(format(&moves), "F R U2");
    }

    #[test]
    fn every_move_round_trips() {
        for m in ALL_MOVES {
            assert_eq!(parse(&m.to_string()).unwrap(), vec![m]);
        }
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse(""), Err(ParseError::Empty));
        assert_eq!(parse("   \t "), Err(ParseError::Empty));
    }

    #[test]
    fn rejects_bad_moves() {
        for bad in [
            "X", "F3", "M", "E", "S", "x", "y", "z", "f", "F2'", "F''", "Fw", "FR", "'", "2",
        ] {
            assert_eq!(
                parse(&format!("R {bad} U")),
                Err(ParseError::InvalidMove(bad.to_string())),
                "{bad}"
            );
        }
    }

    proptest! {
        #[test]
        fn never_panics(s in "\\PC*") {
            let _ = parse(&s);
        }

        #[test]
        fn valid_tokens_always_parse(tokens in prop::collection::vec("[URFDLB]['2]?", 1..50)) {
            let mix = tokens.join(" ");
            prop_assert_eq!(format(&parse(&mix).unwrap()), mix);
        }
    }
}

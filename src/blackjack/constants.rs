use crate::blackjack::{Card, Rank, Suit};

#[rustfmt::skip]
pub const UNSHUFFLED_DECK: [Card; 52] = [
  Card { suit: Suit::Hearts, rank: Rank::Two, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Three, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Four, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Five, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Six, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Seven, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Eight, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Nine, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Ten, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Jack, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Queen, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::King, face_down: false, },
  Card { suit: Suit::Hearts, rank: Rank::Ace, face_down: false, },

  Card { suit: Suit::Diamonds, rank: Rank::Two, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Three, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Four, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Five, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Six, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Seven, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Eight, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Nine, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Ten, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Jack, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Queen, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::King, face_down: false, },
  Card { suit: Suit::Diamonds, rank: Rank::Ace, face_down: false, },

  Card { suit: Suit::Clubs, rank: Rank::Two, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Three, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Four, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Five, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Six, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Seven, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Eight, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Nine, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Ten, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Jack, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Queen, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::King, face_down: false, },
  Card { suit: Suit::Clubs, rank: Rank::Ace, face_down: false, },

  Card { suit: Suit::Spades, rank: Rank::Two, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Three, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Four, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Five, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Six, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Seven, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Eight, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Nine, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Ten, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Jack, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Queen, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::King, face_down: false, },
  Card { suit: Suit::Spades, rank: Rank::Ace, face_down: false, },
];

pub mod basic_strategy_tables {
    pub enum Strategy {
        H,  // hit
        S,  // stand
        D,  // double if possible, otherwise hit
        P,  // split
        DS, // double if possible, otherwise stand
        PH, // split if double down after split is possible, otherwise hit
        RH, // surrender if possible, otherwise hit
    }
    use Strategy::*;
    pub const HARD: [[Strategy; 10]; 13] = [
        /*
        |2  3  4  5  6  7  8  9  10 A */
        [H, H, H, H, H, H, H, H, H, H],    // 5-8
        [H, D, D, D, D, H, H, H, H, H],    // 9
        [D, D, D, D, D, D, D, D, H, H],    // 10
        [D, D, D, D, D, D, D, D, D, D],    // 11
        [H, H, S, S, S, H, H, H, H, H],    // 12
        [S, S, S, S, S, H, H, H, H, H],    // 13
        [S, S, S, S, S, H, H, H, H, H],    // 14
        [S, S, S, S, S, H, H, H, RH, H],   // 15
        [S, S, S, S, S, H, H, RH, RH, RH], // 16
        [S, S, S, S, S, S, S, S, S, S],    // 17
        [S, S, S, S, S, S, S, S, S, S],    // 18
        [S, S, S, S, S, S, S, S, S, S],    // 19
        [S, S, S, S, S, S, S, S, S, S],    // 20
    ];
    pub const SOFT: [[Strategy; 10]; 9] = [
        /*
        |2  3  4  5  6  7  8  9  10 A */
        [H, H, H, H, H, H, H, H, H, H],     // 12
        [H, H, H, D, D, H, H, H, H, H],     // 13
        [H, H, H, D, D, H, H, H, H, H],     // 14
        [H, H, D, D, D, H, H, H, H, H],     // 15
        [H, H, D, D, D, H, H, H, H, H],     // 16
        [H, D, D, D, D, H, H, H, H, H],     // 17
        [S, DS, DS, DS, DS, S, S, H, H, H], // 18
        [S, S, S, S, S, S, S, S, S, S],     // 19
        [S, S, S, S, S, S, S, S, S, S],     // 20
    ];
    pub const SPLIT: [[Strategy; 10]; 10] = [
        /*
        |2  3  4  5  6  7  8  9  10 A */
        [PH, PH, P, P, P, P, H, H, H, H], // 2
        [PH, PH, P, P, P, P, H, H, H, H], // 3
        [H, H, H, PH, PH, H, H, H, H, H], // 4
        [D, D, D, D, D, D, D, D, H, H],   // 5
        [PH, P, P, P, P, H, H, H, H, H],  // 6
        [P, P, P, P, P, P, H, H, H, H],   // 7
        [P, P, P, P, P, P, P, P, P, P],   // 8
        [P, P, P, P, P, S, P, P, S, S],   // 9
        [S, S, S, S, S, S, S, S, S, S],   // 10
        [P, P, P, P, P, P, P, P, P, P],   // Ace
    ];
}

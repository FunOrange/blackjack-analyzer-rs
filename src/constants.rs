use super::blackjack::{Card, FaceValue, Suit};

#[rustfmt::skip]
pub const UNSHUFFLED_DECK: [Card; 52] = [
  Card { suit: Suit::Hearts, face_value: FaceValue::Two, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Three, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Four, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Five, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Six, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Seven, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Eight, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Nine, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Ten, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Jack, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Queen, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::King, face_down: false, },
  Card { suit: Suit::Hearts, face_value: FaceValue::Ace, face_down: false, },

  Card { suit: Suit::Diamonds, face_value: FaceValue::Two, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Three, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Four, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Five, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Six, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Seven, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Eight, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Nine, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Ten, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Jack, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Queen, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::King, face_down: false, },
  Card { suit: Suit::Diamonds, face_value: FaceValue::Ace, face_down: false, },

  Card { suit: Suit::Clubs, face_value: FaceValue::Two, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Three, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Four, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Five, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Six, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Seven, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Eight, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Nine, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Ten, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Jack, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Queen, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::King, face_down: false, },
  Card { suit: Suit::Clubs, face_value: FaceValue::Ace, face_down: false, },

  Card { suit: Suit::Spades, face_value: FaceValue::Two, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Three, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Four, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Five, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Six, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Seven, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Eight, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Nine, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Ten, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Jack, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Queen, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::King, face_down: false, },
  Card { suit: Suit::Spades, face_value: FaceValue::Ace, face_down: false, },
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
    pub const hard: [[Strategy; 10]; 13] = [
        /*
        |2  3  4  5  6  7  8  9  10 A */
        [H, H, H, H, H, H, H, H, H, H], // 5-8
        [H, D, D, D, D, H, H, H, H, H], // 9
        [D, D, D, D, D, D, D, D, H, H], // 10
        [D, D, D, D, D, D, D, D, D, D], // 11
        [H, H, S, S, S, H, H, H, H, H], // 12
        [S, S, S, S, S, H, H, H, H, H], // 13
        [S, S, S, S, S, H, H, H, H, H], // 14
        [S, S, S, S, S, H, H, H, H, H], // 15
        [S, S, S, S, S, H, H, H, H, H], // 16
        [S, S, S, S, S, S, S, S, S, S], // 17
        [S, S, S, S, S, S, S, S, S, S], // 18
        [S, S, S, S, S, S, S, S, S, S], // 19
        [S, S, S, S, S, S, S, S, S, S], // 20
    ];
    pub const soft: [[Strategy; 10]; 9] = [
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
    pub const split: [[Strategy; 10]; 10] = [
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

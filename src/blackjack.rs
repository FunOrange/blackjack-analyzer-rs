use rand::seq::SliceRandom;
use rand::thread_rng;
use ruleset::SplitAces;

use crate::constants::UNSHUFFLED_DECK;
pub mod ruleset;

#[derive(Debug, Clone)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}
#[derive(Debug, Clone)]
pub enum FaceValue {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}
#[derive(Debug, Clone)]
pub struct Card {
    pub suit: Suit,
    pub face_value: FaceValue,
    pub face_down: bool,
}

#[derive(Debug)]
pub enum GameState {
    Dealing,
    PlayerTurn,
    DealerTurn,
    GameOver,
}

#[derive(Debug)]
pub struct BlackjackState {
    pub starting_bet: f32,
    pub shoe: Vec<Card>,
    pub dealer_hand: Vec<Card>,
    pub player_hands: [Vec<Card>; 4],
    pub hand_index: usize,
    pub bets: Vec<f32>,
    pub rules: ruleset::BlackjackRuleset,
    pub state: GameState,
}

pub fn card_value(card: &Card, with_ace_as_11: bool) -> u8 {
    if card.face_down {
        0
    } else {
        match card.face_value {
            FaceValue::Two => 2,
            FaceValue::Three => 3,
            FaceValue::Four => 4,
            FaceValue::Five => 5,
            FaceValue::Six => 6,
            FaceValue::Seven => 7,
            FaceValue::Eight => 8,
            FaceValue::Nine => 9,
            FaceValue::Ten | FaceValue::Jack | FaceValue::Queen | FaceValue::King => 10,
            FaceValue::Ace => {
                if with_ace_as_11 {
                    11
                } else {
                    1
                }
            }
        }
    }
}

pub enum HandValue {
    Hard(u8),
    Soft(u8),
    Blackjack,
}

pub fn format_hand_value(value: &HandValue) -> String {
    match value {
        HandValue::Hard(v) => v.to_string(),
        HandValue::Soft(11) => "Ace".to_string(),
        HandValue::Soft(v) => format!("{}/{}", v - 10, v),
        HandValue::Blackjack => "Blackjack".to_string(),
    }
}

pub fn init_state(starting_bet: f32, rules: ruleset::BlackjackRuleset) -> BlackjackState {
    // todo: shuffle shoe
    let mut shoe: Vec<Card> = UNSHUFFLED_DECK
        .iter()
        .cycle()
        .take(UNSHUFFLED_DECK.len() * 8)
        .cloned()
        .collect();
    shoe.shuffle(&mut thread_rng());
    BlackjackState {
        starting_bet,
        shoe,
        dealer_hand: Vec::new(),
        player_hands: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        hand_index: 0,
        bets: Vec::new(),
        rules,
        state: GameState::Dealing,
    }
}

impl BlackjackState {
    fn hand_value_base(
        &self,
        _hand: &Vec<Card>,
        aces_split: bool,
        include_face_down: bool,
    ) -> HandValue {
        let hand = _hand
            .iter()
            .filter(|c| {
                if include_face_down {
                    true
                } else {
                    !c.face_down
                }
            })
            .collect::<Vec<&Card>>();
        if hand.len() == 2 {
            let card1 = &hand[0].face_value;
            let card2 = &hand[1].face_value;
            let is_blackjack = matches!(
                (card1, card2, &self.rules.ace_and_ten_counts_as_blackjack),
                (FaceValue::Ace, FaceValue::Ten, true)
                    | (FaceValue::Ace, FaceValue::Jack, _)
                    | (FaceValue::Ace, FaceValue::Queen, _)
                    | (FaceValue::Ace, FaceValue::King, _)
                    | (FaceValue::Ten, FaceValue::Ace, true)
                    | (FaceValue::Jack, FaceValue::Ace, _)
                    | (FaceValue::Queen, FaceValue::Ace, _)
                    | (FaceValue::King, FaceValue::Ace, _)
            );
            if is_blackjack {
                return if aces_split && !&self.rules.split_ace_can_be_blackjack {
                    HandValue::Hard(21)
                } else {
                    HandValue::Blackjack
                };
            }
        }
        let has_ace = hand.iter().any(|c| matches!(c.face_value, FaceValue::Ace));
        let low_val: u8 = hand.iter().map(|c| card_value(c, false)).sum();
        let is_soft = has_ace && low_val <= 11;
        if is_soft {
            HandValue::Soft(low_val + 10)
        } else {
            HandValue::Hard(low_val)
        }
    }

    fn player_hand_value(&self, hand: &Vec<Card>, aces_split: bool) -> HandValue {
        self.hand_value_base(hand, aces_split, false)
    }

    fn dealer_hand_value(&self, hand: &Vec<Card>) -> HandValue {
        self.hand_value_base(hand, false, true)
    }

    fn switch_to_split_hand(&self) -> usize {
        match self
            .player_hands
            .iter()
            .skip(self.hand_index + 1)
            .position(|hand| hand.len() == 1)
        {
            Some(index) => self.hand_index + 1 + index,
            None => self.hand_index,
        }
    }

    fn aces_split(&self) -> bool {
        self.player_hands.len() > 1
            && self
                .player_hands
                .iter()
                .all(|hand| matches!(hand[0].face_value, FaceValue::Ace))
    }

    fn player_hand_finished(&self, player_hands: &[Vec<Card>; 4]) -> bool {
        let player_hand = &player_hands[*&self.hand_index];
        let player_hand_value = &self.player_hand_value(player_hand, self.aces_split());
        let is_ace_pair = player_hand.len() == 2
            && player_hand
                .iter()
                .all(|card| matches!(card.face_value, FaceValue::Ace));
        let cannot_resplit_ace = match &self.rules.split_aces {
            SplitAces::NotAllowed | SplitAces::Once => true,
            SplitAces::Twice => player_hands.len() >= 3,
            SplitAces::Thrice => player_hands.len() >= 4,
        };
        let split_ace_finished = if !self.aces_split() {
            false
        } else if *&self.rules.hit_on_split_ace {
            false
        } else if is_ace_pair {
            cannot_resplit_ace
        } else {
            true
        };
        let bust = match player_hand_value {
            HandValue::Hard(n) => *n > 21,
            _ => false,
        };
        let twenty_one = match player_hand_value {
            HandValue::Hard(n) => *n == 21,
            _ => false,
        };
        let soft_twenty_one = match player_hand_value {
            HandValue::Soft(n) => *n == 21,
            _ => false,
        };
        let blackjack = matches!(player_hand_value, HandValue::Blackjack);
        split_ace_finished || bust || twenty_one || soft_twenty_one || blackjack
    }
}

pub enum PlayerAction {
    Hit,
    Stand,
    DoubleDown,
    Split,
}

pub fn next_state(game: BlackjackState, playerAction: Option<PlayerAction>) -> BlackjackState {
    match game.state {
        GameState::Dealing => match (
            game.dealer_hand.as_slice(),
            (
                &game.player_hands[0].as_slice(),
                &game.player_hands[1].as_slice(),
                &game.player_hands[2].as_slice(),
                &game.player_hands[3].as_slice(),
            ),
        ) {
            ([], ([], [], [], [])) => {
                // deal first card to player
                let (player_card, shoe) = game.shoe.split_first().unwrap();
                let mut player_hands = game.player_hands.clone();
                player_hands[0].push(player_card.clone());
                BlackjackState {
                    shoe: shoe.to_vec(),
                    player_hands,
                    ..game
                }
            }
            ([], ([_1], [], [], [])) => {
                // deal second card to dealer
                let (dealer_card, shoe) = game.shoe.split_first().unwrap();
                BlackjackState {
                    shoe: shoe.to_vec(),
                    dealer_hand: vec![dealer_card.clone()],
                    ..game
                }
            }
            ([_2], ([_1], [], [], [])) => {
                // deal third card to player
                let (player_card, shoe) = game.shoe.split_first().unwrap();
                let mut player_hands = game.player_hands.clone();
                player_hands[0].push(player_card.clone());
                BlackjackState {
                    shoe: shoe.to_vec(),
                    player_hands,
                    ..game
                }
            }
            ([_2], ([_1, _3], [], [], [])) => {
                // deal fourth card to dealer (face down)
                let (dealer_card, shoe) = game.shoe.split_first().unwrap();
                let mut dealer_hand = game.dealer_hand.clone();
                dealer_hand.push(Card {
                    face_down: true,
                    ..dealer_card.clone()
                });

                let dealer_hand_value = game.dealer_hand_value(&dealer_hand);
                if game.rules.dealer_peeks && matches!(dealer_hand_value, HandValue::Blackjack) {
                    BlackjackState {
                        shoe: shoe.to_vec(),
                        dealer_hand,
                        state: GameState::GameOver,
                        ..game
                    }
                } else {
                    match game.player_hand_value(&game.player_hands[0], false) {
                        HandValue::Blackjack | HandValue::Hard(21) => BlackjackState {
                            shoe: shoe.to_vec(),
                            dealer_hand,
                            state: GameState::DealerTurn, // dealer could still have 21/blackjack
                            ..game
                        },
                        _ => BlackjackState {
                            shoe: shoe.to_vec(),
                            dealer_hand,
                            state: GameState::PlayerTurn,
                            ..game
                        },
                    }
                }
            }
            (_, (_, [_card], _, _)) | (_, (_, _, [_card], _)) | (_, (_, _, _, [_card])) => {
                // player just split, deal 1 card
                // note: bust impossible no need to check
                let (player_card, shoe) = game.shoe.split_first().unwrap();
                let mut player_hands = game.player_hands.clone();
                player_hands[game.hand_index].push(player_card.clone());
                let player_hand_finished = game.player_hand_finished(&player_hands);
                let hand_index = if player_hand_finished {
                    game.switch_to_split_hand()
                } else {
                    game.hand_index
                };
                let switching_to_split_hand = hand_index != game.hand_index;
                let state = if switching_to_split_hand {
                    GameState::Dealing
                } else if player_hand_finished && !switching_to_split_hand {
                    GameState::DealerTurn
                } else {
                    GameState::PlayerTurn
                };
                BlackjackState {
                    shoe: shoe.to_vec(),
                    player_hands,
                    hand_index,
                    state,
                    ..game
                }
            }
            _ => {
                panic!("Unreachable code: {:?}", game);
            }
        },
        GameState::PlayerTurn => game,
        GameState::DealerTurn => game,
        GameState::GameOver => game,
    }
}

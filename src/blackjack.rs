pub mod ruleset;
use crate::constants::UNSHUFFLED_DECK;
use crate::terminal::yellow;
use core::panic;
use rand::seq::SliceRandom;
use rand::thread_rng;
use ruleset::BlackjackRuleset;

#[derive(Debug, Clone)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}
impl ToString for Suit {
    fn to_string(&self) -> String {
        match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
        .to_string()
    }
}
#[derive(Debug, Clone, PartialEq)]
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
impl ToString for FaceValue {
    fn to_string(&self) -> String {
        match self {
            FaceValue::Two => "2",
            FaceValue::Three => "3",
            FaceValue::Four => "4",
            FaceValue::Five => "5",
            FaceValue::Six => "6",
            FaceValue::Seven => "7",
            FaceValue::Eight => "8",
            FaceValue::Nine => "9",
            FaceValue::Ten => "10",
            FaceValue::Jack => "J",
            FaceValue::Queen => "Q",
            FaceValue::King => "K",
            FaceValue::Ace => "A",
        }
        .to_string()
    }
}
#[derive(Debug, Clone)]
pub struct Card {
    pub suit: Suit,
    pub face_value: FaceValue,
    pub face_down: bool,
}
impl ToString for Card {
    fn to_string(&self) -> String {
        if self.face_down {
            "?".to_string()
        } else {
            format!("{}{}", self.face_value.to_string(), self.suit.to_string())
        }
    }
}

#[derive(Debug, Clone)]
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
    pub player_hands: Vec<Vec<Card>>,
    pub hand_index: usize,
    pub bets: Vec<f32>,
    pub rules: BlackjackRuleset,
    pub state: GameState,
}
impl Clone for BlackjackState {
    fn clone(&self) -> Self {
        BlackjackState {
            starting_bet: self.starting_bet,
            shoe: self.shoe.clone(),
            dealer_hand: self.dealer_hand.clone(),
            player_hands: self.player_hands.clone(),
            hand_index: self.hand_index,
            bets: self.bets.clone(),
            rules: self.rules.clone(),
            state: self.state.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    Hit,
    Stand,
    DoubleDown,
    Split,
}
impl PartialEq for PlayerAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PlayerAction::Hit, PlayerAction::Hit)
            | (PlayerAction::Stand, PlayerAction::Stand)
            | (PlayerAction::DoubleDown, PlayerAction::DoubleDown)
            | (PlayerAction::Split, PlayerAction::Split) => true,
            _ => false,
        }
    }
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

#[derive(Copy, Clone)]
pub enum HandValue {
    Hard(u8),
    Soft(u8),
    Blackjack,
}
use HandValue::*;

pub fn format_hand_value(value: &HandValue) -> String {
    match value {
        Hard(v) => v.to_string(),
        Soft(11) => "Ace".to_string(),
        Soft(v) => format!("{}/{}", v - 10, v),
        Blackjack => "Blackjack".to_string(),
    }
}

pub fn init_state(starting_bet: f32, rules: BlackjackRuleset) -> BlackjackState {
    let mut shoe: Vec<Card> = UNSHUFFLED_DECK
        .iter()
        .cycle()
        .take(UNSHUFFLED_DECK.len() * 8)
        .cloned()
        .collect();
    shoe.shuffle(&mut thread_rng());

    #[rustfmt::skip]
    let mut debug_shoe_starting_pairs = vec![
        Card { suit: Suit::Hearts, face_value: FaceValue::Two, face_down: false },
        Card { suit: Suit::Hearts, face_value: FaceValue::Two, face_down: false },
        Card { suit: Suit::Hearts, face_value: FaceValue::Two, face_down: false },
        Card { suit: Suit::Hearts, face_value: FaceValue::Two, face_down: false },
    ];
    debug_shoe_starting_pairs.extend(shoe);
    BlackjackState {
        starting_bet,
        shoe: debug_shoe_starting_pairs,
        dealer_hand: Vec::new(),
        player_hands: vec![Vec::new()],
        hand_index: 0,
        bets: vec![starting_bet],
        rules,
        state: GameState::Dealing,
    }
}

fn bust(hand: &Vec<Card>) -> bool {
    let value: u8 = hand.iter().map(|c| card_value(c, false)).sum();
    value > 21
}

pub enum WinReason {
    DealerBust,
    HigherHand,
    Blackjack, // technically redundant but useful for displaying to user
}
pub enum LossReason {
    Bust,
    LowerHand,
    DealerBlackjack, // technically redundant but useful for displaying to user
}
pub enum HandOutcome {
    Win(WinReason),
    Lose(LossReason),
    Push,
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
                    Hard(21)
                } else {
                    Blackjack
                };
            }
        }
        let has_ace = hand.iter().any(|c| matches!(c.face_value, FaceValue::Ace));
        let low_val: u8 = hand.iter().map(|c| card_value(c, false)).sum();
        let is_soft = has_ace && low_val <= 11;
        if is_soft {
            Soft(low_val + 10)
        } else {
            Hard(low_val)
        }
    }

    fn player_hand_value(&self, hand: &Vec<Card>, aces_split: bool) -> HandValue {
        self.hand_value_base(hand, aces_split, false)
    }

    fn dealer_hand_value(&self, hand: &Vec<Card>) -> HandValue {
        self.hand_value_base(hand, false, true)
    }

    fn aces_split(&self, player_hands: &Vec<Vec<Card>>) -> bool {
        player_hands.len() > 1
            && player_hands
                .iter()
                .all(|hand| matches!(hand[0].face_value, FaceValue::Ace))
    }

    fn next_split_hand_index(&self, player_hands: &Vec<Vec<Card>>) -> usize {
        match player_hands
            .iter()
            .skip(self.hand_index + 1)
            .position(|hand| hand.len() == 1)
        {
            Some(index) => self.hand_index + 1 + index,
            None => self.hand_index,
        }
    }

    fn player_split_aces(&self, player_hands: &Vec<Vec<Card>>) -> bool {
        player_hands.len() >= 2
            && player_hands
                .iter()
                .all(|hand| matches!(hand[0].face_value, FaceValue::Ace))
    }

    fn player_hand_finished(&self, player_hands: &Vec<Vec<Card>>) -> bool {
        let player_hand = &player_hands[self.hand_index];
        let player_hand_value =
            self.player_hand_value(player_hand, self.player_split_aces(player_hands));
        let pair_of_aces = player_hand.len() == 2
            && player_hand
                .iter()
                .all(|card| matches!(card.face_value, FaceValue::Ace));
        let cannot_resplit_ace = match &self.rules.split_aces {
            ruleset::SplitAces::NotAllowed | ruleset::SplitAces::Once => true, // cannot split ace
            ruleset::SplitAces::Twice => self.player_hands.len() >= 3,
            ruleset::SplitAces::Thrice => self.player_hands.len() >= 4,
        };
        let split_ace_finished = if !self.player_split_aces(player_hands) {
            false
        } else if self.rules.hit_on_split_ace {
            false // this hand is not finished; player can hit or split aces
        } else if pair_of_aces {
            cannot_resplit_ace // player may not hit but might be able to split aces
        } else {
            true // this hand is finished (eg. A, 5)
        };
        let twenty_one = match player_hand_value {
            Hard(n) => n == 21,
            _ => false,
        };
        let soft_twenty_one = match player_hand_value {
            Soft(n) => n == 21,
            _ => false,
        };
        let blackjack = matches!(player_hand_value, Blackjack);
        bust(player_hand) || split_ace_finished || twenty_one || soft_twenty_one || blackjack
    }

    pub fn allowed_actions(&self) -> Vec<PlayerAction> {
        if !matches!(&self.state, GameState::PlayerTurn) {
            panic!("Invalid state: {:?}", &self.state);
        }

        let player_hand = &self.player_hands[self.hand_index];
        let player_hand_value =
            self.player_hand_value(player_hand, self.player_split_aces(&self.player_hands));
        if self.player_hand_finished(&self.player_hands) {
            self.print_game_state();
            panic!("Player hand is finished; no allowed actions on this hand.");
        }

        let can_hit = if self.rules.hit_on_split_ace {
            true
        } else {
            !self.aces_split(&self.player_hands)
        };

        let can_split = {
            let is_pair =
                player_hand.len() == 2 && player_hand[0].face_value == player_hand[1].face_value;
            let can_split_aces = {
                let num_aces_split = if self.aces_split(&self.player_hands) {
                    self.player_hands.len()
                } else {
                    0
                };
                match &self.rules.split_aces {
                    ruleset::SplitAces::NotAllowed => false,
                    ruleset::SplitAces::Once => num_aces_split < 1,
                    ruleset::SplitAces::Twice => num_aces_split < 2,
                    ruleset::SplitAces::Thrice => num_aces_split < 3,
                }
            };
            let house_rule_satisfied = match self.rules.max_hands_after_split {
                ruleset::MaxHandsAfterSplit::One => false, // can never split
                ruleset::MaxHandsAfterSplit::Two => self.player_hands.len() < 2,
                ruleset::MaxHandsAfterSplit::Three => self.player_hands.len() < 3,
                ruleset::MaxHandsAfterSplit::Four => self.player_hands.len() < 4,
            };
            is_pair
                && house_rule_satisfied
                && match player_hand[0].face_value {
                    FaceValue::Ace => can_split_aces,
                    _ => true,
                }
        };
        let can_double_down = {
            let house_rule_satisfied = match &self.rules.double_down_on {
                ruleset::DoubleDownOn::Any => true,
                ruleset::DoubleDownOn::NineTenEleven => {
                    matches!(player_hand_value, Hard(9) | Hard(10) | Hard(11))
                }
                ruleset::DoubleDownOn::TenEleven => {
                    matches!(player_hand_value, Hard(10) | Hard(11))
                }
            };
            player_hand.len() == 2 && house_rule_satisfied
        };

        let mut allowed_actions: Vec<PlayerAction> = vec![];
        if can_hit {
            allowed_actions.push(PlayerAction::Hit);
        }
        allowed_actions.push(PlayerAction::Stand);
        if can_split {
            allowed_actions.push(PlayerAction::Split);
        }
        if can_double_down {
            allowed_actions.push(PlayerAction::DoubleDown);
        }
        allowed_actions
    }

    pub fn print_game_state(&self) {
        print!("Dealer hand:");
        for card in &self.dealer_hand {
            print!(
                " {}",
                match card.face_down {
                    true => "■".to_string(),
                    false => card.face_value.to_string(),
                }
            );
        }
        println!();
        for (i, hand) in self
            .player_hands
            .iter()
            .filter(|&h| !h.is_empty())
            .enumerate()
        {
            print!("Player hand:");
            for card in hand {
                print!(" {}", card.face_value.to_string());
            }
            if i == self.hand_index {
                print!("{}", yellow(" ←"));
            }
            println!();
        }
    }

    pub fn next_state(&self, player_action: Option<PlayerAction>) -> BlackjackState {
        match self.state {
            GameState::Dealing => match (
                self.dealer_hand.as_slice(),
                (
                    &self.player_hands.get(0).unwrap_or(&vec![]).as_slice(),
                    &self.player_hands.get(1).unwrap_or(&vec![]).as_slice(),
                    &self.player_hands.get(2).unwrap_or(&vec![]).as_slice(),
                    &self.player_hands.get(3).unwrap_or(&vec![]).as_slice(),
                ),
            ) {
                ([], ([], [], [], [])) => {
                    // deal first card to player
                    let (player_card, shoe) = self.shoe.split_first().unwrap();
                    let mut player_hands = self.player_hands.clone();
                    player_hands[0].push(player_card.clone());
                    BlackjackState {
                        shoe: shoe.to_vec(),
                        player_hands,
                        ..self.clone()
                    }
                }
                ([], ([_1], [], [], [])) => {
                    // deal second card to dealer
                    let (dealer_card, shoe) = self.shoe.split_first().unwrap();
                    BlackjackState {
                        shoe: shoe.to_vec(),
                        dealer_hand: vec![dealer_card.clone()],
                        ..self.clone()
                    }
                }
                ([_2], ([_1], [], [], [])) => {
                    // deal third card to player
                    let (player_card, shoe) = self.shoe.split_first().unwrap();
                    let mut player_hands = self.player_hands.clone();
                    player_hands[0].push(player_card.clone());
                    BlackjackState {
                        shoe: shoe.to_vec(),
                        player_hands,
                        ..self.clone()
                    }
                }
                ([_2], ([_1, _3], [], [], [])) => {
                    // deal fourth card to dealer (face down)
                    let (dealer_card, shoe) = self.shoe.split_first().unwrap();
                    let mut dealer_hand = self.dealer_hand.clone();
                    dealer_hand.push(Card {
                        face_down: true,
                        ..dealer_card.clone()
                    });

                    let dealer_hand_value = self.dealer_hand_value(&dealer_hand);
                    if self.rules.dealer_peeks && matches!(dealer_hand_value, Blackjack) {
                        BlackjackState {
                            shoe: shoe.to_vec(),
                            dealer_hand,
                            state: GameState::GameOver,
                            ..self.clone()
                        }
                    } else {
                        match self.player_hand_value(&self.player_hands[0], false) {
                            Blackjack | Hard(21) => BlackjackState {
                                shoe: shoe.to_vec(),
                                dealer_hand,
                                state: GameState::DealerTurn, // dealer could still have 21/blackjack
                                ..self.clone()
                            },
                            _ => BlackjackState {
                                shoe: shoe.to_vec(),
                                dealer_hand,
                                state: GameState::PlayerTurn,
                                ..self.clone()
                            },
                        }
                    }
                }
                (_, (_, [_card], _, _)) | (_, (_, _, [_card], _)) | (_, (_, _, _, [_card])) => {
                    // player just split, deal 1 card
                    // note: bust impossible no need to check
                    let (player_card, shoe) = self.shoe.split_first().unwrap();
                    let mut player_hands = self.player_hands.clone();
                    player_hands[self.hand_index].push(player_card.clone());
                    let player_hand_finished = self.player_hand_finished(&player_hands);
                    let hand_index = match player_hand_finished {
                        true => self.next_split_hand_index(&player_hands),
                        false => self.hand_index,
                    };
                    let switching_to_split_hand = hand_index != self.hand_index;
                    let state = if player_hands.iter().all(|hand| {
                        let player_hand_value =
                            self.player_hand_value(&hand, self.player_split_aces(&player_hands));
                        let bust = match player_hand_value {
                            Hard(n) => n > 21,
                            _ => false,
                        };
                        bust
                    }) {
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
                        ..self.clone()
                    }
                }
                _ => {
                    panic!("Unreachable code: {:?}", self);
                }
            },
            GameState::PlayerTurn => {
                let allowed_actions = self.allowed_actions();
                let player_action = player_action.unwrap();
                if !allowed_actions.contains(&player_action) {
                    self.print_game_state();
                    panic!(
                        "Invalid action: {:?}. Valid actions are {:?}",
                        player_action, allowed_actions
                    );
                }
                match player_action {
                    PlayerAction::Hit => {
                        let (player_card, shoe) = self.shoe.split_first().unwrap();
                        let mut player_hands = self.player_hands.clone();
                        player_hands[self.hand_index].push(player_card.clone());
                        let player_hand_finished = self.player_hand_finished(&player_hands);
                        let hand_index = if player_hand_finished {
                            self.next_split_hand_index(&player_hands)
                        } else {
                            self.hand_index
                        };
                        let switching_to_split_hand = hand_index != self.hand_index;
                        let state = if player_hands.iter().all(|hand| bust(hand)) {
                            GameState::GameOver
                        } else {
                            match (player_hand_finished, switching_to_split_hand) {
                                (false, false) => GameState::PlayerTurn, // keep playing this hand
                                (true, false) => GameState::DealerTurn,
                                (_, true) => GameState::Dealing, // deal card to next split hand
                            }
                        };
                        BlackjackState {
                            shoe: shoe.to_vec(),
                            player_hands,
                            hand_index,
                            state,
                            ..self.clone()
                        }
                    }
                    PlayerAction::Stand => {
                        let hand_index = self.next_split_hand_index(&self.player_hands);
                        let switching_to_split_hand = hand_index != self.hand_index;
                        match switching_to_split_hand {
                            true => BlackjackState {
                                hand_index,
                                state: GameState::Dealing,
                                ..self.clone()
                            },
                            false => BlackjackState {
                                state: GameState::DealerTurn,
                                ..self.clone()
                            },
                        }
                    }
                    PlayerAction::DoubleDown => {
                        let mut bets = self.bets.clone();
                        bets[self.hand_index] *= 2.0;

                        let (player_card, shoe) = self.shoe.split_first().unwrap();
                        let mut player_hands = self.player_hands.clone();
                        player_hands[self.hand_index].push(player_card.clone());
                        let hand_index = self.next_split_hand_index(&player_hands);
                        let switching_to_split_hand = hand_index != self.hand_index;
                        let state = if player_hands.iter().all(|hand| bust(hand)) {
                            GameState::GameOver
                        } else if switching_to_split_hand {
                            GameState::PlayerTurn
                        } else {
                            GameState::DealerTurn
                        };
                        BlackjackState {
                            bets,
                            shoe: shoe.to_vec(),
                            player_hands,
                            hand_index,
                            state,
                            ..self.clone()
                        }
                    }
                    PlayerAction::Split => {
                        let mut bets = self.bets.clone();
                        bets.push(self.starting_bet);

                        let mut player_hands = self.player_hands.clone();
                        let card2 = player_hands[self.hand_index].pop();
                        player_hands.push(vec![card2.unwrap()]);
                        BlackjackState {
                            bets,
                            player_hands,
                            state: GameState::Dealing,
                            ..self.clone()
                        }
                    }
                }
            }
            GameState::DealerTurn => {
                let dealer_should_stand =
                    |dealer_hand: &Vec<Card>| match self.dealer_hand_value(&dealer_hand) {
                        Soft(n) if n >= 17 => self.rules.dealer_stands_on_all_17,
                        Hard(n) if n >= 17 => true,
                        Blackjack => true,
                        _ => false,
                    };
                if dealer_should_stand(&self.dealer_hand) {
                    BlackjackState {
                        state: GameState::GameOver,
                        ..self.clone()
                    }
                } else {
                    // dealer hits
                    let (dealer_hand, shoe) = {
                        if self.dealer_hand[1].face_down {
                            let mut dealer_hand = self.dealer_hand.clone();
                            dealer_hand[1].face_down = false;
                            (dealer_hand, self.shoe.clone())
                        } else {
                            let (dealer_card, shoe) = self.shoe.split_first().unwrap();
                            let mut dealer_hand = self.dealer_hand.clone();
                            dealer_hand.push(dealer_card.clone());
                            (dealer_hand, shoe.to_vec())
                        }
                    };
                    let state = {
                        let all_blackjacks = self.player_hands.iter().all(|hand| {
                            let aces_were_split = self.player_split_aces(&self.player_hands);
                            let player_hand_value = self.player_hand_value(hand, aces_were_split);
                            matches!(player_hand_value, Blackjack)
                        });
                        if all_blackjacks {
                            // dealer has now revealed face down card and is up against all blackjacks
                            // no need to play out the hand
                            GameState::GameOver
                        } else if dealer_should_stand(&dealer_hand) {
                            GameState::GameOver
                        } else {
                            GameState::DealerTurn
                        }
                    };
                    BlackjackState {
                        shoe,
                        dealer_hand,
                        state,
                        ..self.clone()
                    }
                }
            }
            GameState::GameOver => {
                panic!("Game is over; no more actions allowed.");
            }
        }
    }

    pub fn player_hand_outcomes(&self) -> Vec<HandOutcome> {
        if !matches!(&self.state, GameState::GameOver) {
            panic!("Game is not over; cannot determine outcomes.");
        }
        self.player_hands
            .iter()
            .map(|hand| {
                let player_hand_value =
                    self.player_hand_value(hand, self.player_split_aces(&self.player_hands));
                let dealer_hand_value = self.dealer_hand_value(&self.dealer_hand);
                match (player_hand_value, dealer_hand_value) {
                    (Blackjack, Blackjack) => HandOutcome::Push,
                    (Blackjack, _) => HandOutcome::Win(WinReason::Blackjack),
                    (_, Blackjack) => HandOutcome::Lose(LossReason::DealerBlackjack),
                    _ => {
                        fn to_number(value: &HandValue) -> u8 {
                            match value {
                                Hard(n) => *n,
                                Soft(n) => *n,
                                Blackjack => panic!("Unreachable code"),
                            }
                        }
                        let player_number = to_number(&player_hand_value);
                        let dealer_number = to_number(&dealer_hand_value);
                        if player_number > 21 {
                            HandOutcome::Lose(LossReason::Bust)
                        } else if dealer_number > 21 {
                            HandOutcome::Win(WinReason::DealerBust)
                        } else if player_number > dealer_number {
                            HandOutcome::Win(WinReason::HigherHand)
                        } else if player_number < dealer_number {
                            HandOutcome::Lose(LossReason::LowerHand)
                        } else {
                            HandOutcome::Push
                        }
                    }
                }
            })
            .collect()
    }
}

pub mod ruleset;
use crate::constants::basic_strategy_tables;
use crate::constants::basic_strategy_tables::Strategy;
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

#[derive(Debug, Copy, Clone)]
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
    Surrender,
}
impl PartialEq for PlayerAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PlayerAction::Hit, PlayerAction::Hit)
            | (PlayerAction::Stand, PlayerAction::Stand)
            | (PlayerAction::DoubleDown, PlayerAction::DoubleDown)
            | (PlayerAction::Split, PlayerAction::Split)
            | (PlayerAction::Surrender, PlayerAction::Surrender) => true,
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

const NUM_DECKS: usize = 8; // 8 gives huge increase to code execution speed for some reason
pub fn init_state(starting_bet: f32, rules: BlackjackRuleset) -> BlackjackState {
    let mut shoe: Vec<Card> = Vec::with_capacity(UNSHUFFLED_DECK.len() * NUM_DECKS);
    for _ in 0..NUM_DECKS {
        shoe.extend(UNSHUFFLED_DECK.iter().cloned());
    }
    shoe.shuffle(&mut thread_rng());

    // #[rustfmt::skip]
    // let debug_start = vec![
    //     Card { suit: Suit::Hearts, face_value: FaceValue::Six, face_down: false },
    //     Card { suit: Suit::Hearts, face_value: FaceValue::Ace, face_down: false },
    //     Card { suit: Suit::Hearts, face_value: FaceValue::King, face_down: false },
    //     Card { suit: Suit::Hearts, face_value: FaceValue::Ace, face_down: false },
    //     Card { suit: Suit::Hearts, face_value: FaceValue::Four, face_down: false },
    //     Card { suit: Suit::Hearts, face_value: FaceValue::Ace, face_down: false }, // first card
    // ];
    // shoe.extend(debug_start);
    let mut player_hands = Vec::with_capacity(4);
    player_hands.push(Vec::with_capacity(8));
    BlackjackState {
        starting_bet,
        shoe,
        dealer_hand: Vec::with_capacity(8),
        player_hands,
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
    Won(WinReason),
    Lost(LossReason),
    Push,
    Surrendered,
}

impl BlackjackState {
    fn hand_value_base(&self, _hand: &Vec<Card>, aces_split: bool) -> HandValue {
        let hand = _hand
            .iter()
            .filter(|c| !c.face_down)
            .collect::<Vec<&Card>>();
        if hand.len() == 2 {
            let card1 = &hand[0].face_value;
            let card2 = &hand[1].face_value;
            let is_blackjack = match (card1, card2, &self.rules.ace_and_ten_counts_as_blackjack) {
                (FaceValue::Ace, FaceValue::Ten, true) => true,
                (FaceValue::Ace, FaceValue::Jack, _) => true,
                (FaceValue::Ace, FaceValue::Queen, _) => true,
                (FaceValue::Ace, FaceValue::King, _) => true,
                (FaceValue::Ten, FaceValue::Ace, true) => true,
                (FaceValue::Jack, FaceValue::Ace, _) => true,
                (FaceValue::Queen, FaceValue::Ace, _) => true,
                (FaceValue::King, FaceValue::Ace, _) => true,
                _ => false,
            };
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
        match has_ace && low_val <= 11 {
            true => Soft(low_val + 10),
            false => Hard(low_val),
        }
    }

    fn player_hand_value(&self, hand: &Vec<Card>, aces_split: bool) -> HandValue {
        self.hand_value_base(hand, aces_split)
    }

    fn dealer_hand_value(&self, hand: &Vec<Card>) -> HandValue {
        self.hand_value_base(hand, false)
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
            player_hand.len() == 2
                && house_rule_satisfied
                && !player_hand
                    .iter()
                    .all(|c| matches!(c.face_value, FaceValue::Ace))
        };

        let can_surrender = self.rules.surrender
            && self.player_hands.len() == 1
            && self.player_hands[0].len() == 2
            && self.dealer_hand[1].face_down;

        let mut allowed_actions: Vec<PlayerAction> = Vec::with_capacity(4);
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
        if can_surrender {
            allowed_actions.push(PlayerAction::Surrender);
        }
        allowed_actions
    }

    pub fn get_optimal_move(&self) -> PlayerAction {
        let allowed_actions = self.allowed_actions();
        let dealer_upcard = &self.dealer_hand[1];
        let dealer_upcard = card_value(dealer_upcard, true);
        let player_hand = &self.player_hands[self.hand_index];
        let can_split = allowed_actions.contains(&PlayerAction::Split);
        let strategy = if can_split {
            let card_value = card_value(&player_hand[0], true);
            &basic_strategy_tables::split[card_value as usize - 2][dealer_upcard as usize - 2]
        } else {
            match self.player_hand_value(&self.player_hands[self.hand_index], false) {
                Hard(n) => &basic_strategy_tables::hard[n as usize - 5][dealer_upcard as usize - 2],
                Soft(n) => {
                    &basic_strategy_tables::soft[n as usize - 13][dealer_upcard as usize - 2]
                }
                Blackjack => {
                    panic!("Unreachable code.")
                }
            }
        };
        match strategy {
            Strategy::H => match allowed_actions.contains(&PlayerAction::Hit) {
                true => PlayerAction::Hit,
                false => PlayerAction::Stand,
            },
            Strategy::S => PlayerAction::Stand,
            Strategy::D => match allowed_actions.contains(&PlayerAction::DoubleDown) {
                true => PlayerAction::DoubleDown,
                false => PlayerAction::Hit,
            },
            Strategy::P => PlayerAction::Split,
            Strategy::DS => match allowed_actions.contains(&PlayerAction::DoubleDown) {
                true => PlayerAction::DoubleDown,
                false => PlayerAction::Stand,
            },
            Strategy::PH => match &self.rules.double_after_split {
                true => PlayerAction::Split,
                false => PlayerAction::Hit,
            },
            Strategy::RH => match allowed_actions.contains(&PlayerAction::Surrender) {
                true => PlayerAction::Surrender,
                false => PlayerAction::Hit,
            },
        }
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

    pub fn next_state(&mut self, player_action: Option<PlayerAction>) -> () {
        match self.state {
            GameState::Dealing => match (
                self.dealer_hand.len(),
                (
                    self.player_hands.get(0).unwrap_or(&vec![]).len(),
                    self.player_hands.get(1).unwrap_or(&vec![]).len(),
                    self.player_hands.get(2).unwrap_or(&vec![]).len(),
                    self.player_hands.get(3).unwrap_or(&vec![]).len(),
                ),
            ) {
                (0, (0, 0, 0, 0)) => {
                    // deal first card to player
                    let player_card = self.shoe.pop().unwrap();
                    self.player_hands[0].push(player_card);
                }
                (0, (1, 0, 0, 0)) => {
                    // deal second card to dealer
                    let dealer_card = self.shoe.pop().unwrap();
                    self.dealer_hand.push(dealer_card);
                }
                (1, (1, 0, 0, 0)) => {
                    // deal third card to player
                    let player_card = self.shoe.pop().unwrap();
                    self.player_hands[0].push(player_card);
                }
                (1, (2, 0, 0, 0)) => {
                    // deal fourth card to dealer (face down)
                    let dealer_card = self.shoe.pop().unwrap();
                    self.dealer_hand.push(Card {
                        face_down: true,
                        ..dealer_card
                    });
                    let dealer_hand_value = self.dealer_hand_value(&self.dealer_hand);
                    if self.rules.dealer_peeks && matches!(dealer_hand_value, Blackjack) {
                        self.state = GameState::GameOver;
                    } else {
                        match self.player_hand_value(&self.player_hands[0], false) {
                            Blackjack | Hard(21) => {
                                self.state = GameState::DealerTurn;
                            }
                            _ => {
                                self.state = GameState::PlayerTurn;
                            }
                        }
                    }
                }
                (_, (_, 1, _, _)) | (_, (_, _, 1, _)) | (_, (_, _, _, 1)) => {
                    // player just split, deal 1 card
                    // note: bust impossible no need to check
                    let player_card = self.shoe.pop().unwrap();
                    self.player_hands[self.hand_index].push(player_card);
                    let player_hand_finished = self.player_hand_finished(&self.player_hands);
                    let hand_index = match player_hand_finished {
                        true => self.next_split_hand_index(&self.player_hands),
                        false => self.hand_index,
                    };
                    let switching_to_split_hand = hand_index != self.hand_index;
                    let state = if switching_to_split_hand {
                        GameState::Dealing
                    } else if player_hand_finished && !switching_to_split_hand {
                        GameState::DealerTurn
                    } else {
                        GameState::PlayerTurn
                    };
                    self.hand_index = hand_index;
                    self.state = state;
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
                        let player_card = self.shoe.pop().unwrap();
                        self.player_hands[self.hand_index].push(player_card);
                        let player_hand_finished = self.player_hand_finished(&self.player_hands);
                        let hand_index = if player_hand_finished {
                            self.next_split_hand_index(&self.player_hands)
                        } else {
                            self.hand_index
                        };
                        let switching_to_split_hand = hand_index != self.hand_index;
                        let state = if self.player_hands.iter().all(|hand| bust(hand)) {
                            GameState::GameOver
                        } else {
                            match (player_hand_finished, switching_to_split_hand) {
                                (false, false) => GameState::PlayerTurn, // keep playing this hand
                                (true, false) => GameState::DealerTurn,
                                (_, true) => GameState::Dealing, // deal card to next split hand
                            }
                        };
                        self.hand_index = hand_index;
                        self.state = state;
                    }
                    PlayerAction::Stand => {
                        let hand_index = self.next_split_hand_index(&self.player_hands);
                        let switching_to_split_hand = hand_index != self.hand_index;
                        match switching_to_split_hand {
                            true => {
                                self.hand_index = hand_index;
                                self.state = GameState::Dealing;
                            }
                            false => {
                                self.state = GameState::DealerTurn;
                            }
                        }
                    }
                    PlayerAction::DoubleDown => {
                        self.bets[self.hand_index] *= 2.0;

                        let player_card = self.shoe.pop().unwrap();
                        self.player_hands[self.hand_index].push(player_card);
                        let hand_index = self.next_split_hand_index(&self.player_hands);
                        let switching_to_split_hand = hand_index != self.hand_index;
                        self.hand_index = hand_index;
                        self.state = if self.player_hands.iter().all(|hand| bust(hand)) {
                            GameState::GameOver
                        } else if switching_to_split_hand {
                            GameState::PlayerTurn
                        } else {
                            GameState::DealerTurn
                        };
                    }
                    PlayerAction::Split => {
                        self.bets.push(self.starting_bet);
                        let card2 = self.player_hands[self.hand_index].pop().unwrap();
                        let mut new_hand = Vec::with_capacity(8);
                        new_hand.push(card2);
                        self.player_hands.push(new_hand);
                        self.state = GameState::Dealing;
                    }
                    PlayerAction::Surrender => {
                        self.state = GameState::GameOver;
                    }
                }
            }
            GameState::DealerTurn => {
                fn dealer_should_stand(game: &BlackjackState) -> bool {
                    match game.dealer_hand_value(&game.dealer_hand) {
                        Soft(n) if n >= 17 => game.rules.dealer_stands_on_all_17,
                        Hard(n) if n >= 17 => true,
                        Blackjack => true,
                        _ => false,
                    }
                };
                if dealer_should_stand(&self) {
                    self.state = GameState::GameOver;
                } else {
                    // dealer hits
                    {
                        if self.dealer_hand[1].face_down {
                            self.dealer_hand[1].face_down = false;
                        } else {
                            let dealer_card = self.shoe.pop().unwrap();
                            self.dealer_hand.push(dealer_card);
                        }
                    };
                    self.state = {
                        let all_blackjacks = self.player_hands.iter().all(|hand| {
                            let aces_were_split = self.player_split_aces(&self.player_hands);
                            let player_hand_value = self.player_hand_value(hand, aces_were_split);
                            matches!(player_hand_value, Blackjack)
                        });
                        if all_blackjacks {
                            // dealer has now revealed face down card and is up against all blackjacks
                            // no need to play out the hand
                            GameState::GameOver
                        } else if dealer_should_stand(&self) {
                            GameState::GameOver
                        } else {
                            GameState::DealerTurn
                        }
                    };
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
        // check for surrender
        if self.player_hands.len() == 1
            && self.player_hands[0].len() == 2
            && self.dealer_hand[1].face_down
        {
            return vec![HandOutcome::Surrendered];
        }

        self.player_hands
            .iter()
            .map(|hand| {
                let player_hand_value =
                    self.player_hand_value(hand, self.player_split_aces(&self.player_hands));
                let dealer_hand_value = self.dealer_hand_value(&self.dealer_hand);
                match (player_hand_value, dealer_hand_value) {
                    (Blackjack, Blackjack) => HandOutcome::Push,
                    (Blackjack, _) => HandOutcome::Won(WinReason::Blackjack),
                    (_, Blackjack) => HandOutcome::Lost(LossReason::DealerBlackjack),
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
                            HandOutcome::Lost(LossReason::Bust)
                        } else if dealer_number > 21 {
                            HandOutcome::Won(WinReason::DealerBust)
                        } else if player_number > dealer_number {
                            HandOutcome::Won(WinReason::HigherHand)
                        } else if player_number < dealer_number {
                            HandOutcome::Lost(LossReason::LowerHand)
                        } else {
                            HandOutcome::Push
                        }
                    }
                }
            })
            .collect()
    }
}

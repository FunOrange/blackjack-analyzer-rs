use std::collections::HashMap;

use rand::{prelude::SliceRandom, thread_rng};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::blackjack::{
    card_value, constants::UNSHUFFLED_DECK, ruleset::*, BlackjackState, Card, GameState, HandValue,
    PlayerAction, Rank,
};

#[wasm_bindgen]
pub fn init_state(starting_bet: f32, rules: JsValue) -> JsValue {
    let rules: BlackjackRuleset = serde_wasm_bindgen::from_value(rules).unwrap();
    let game = crate::blackjack::init_state(starting_bet, rules);
    serde_wasm_bindgen::to_value(&game).unwrap()
}

#[wasm_bindgen]
pub fn next_state(game: JsValue, action: JsValue) -> JsValue {
    let mut game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    let action: Option<PlayerAction> = if action == JsValue::UNDEFINED {
        None
    } else {
        let action: PlayerAction = serde_wasm_bindgen::from_value(action).unwrap();
        Some(action)
    };
    game.next_state(action);
    serde_wasm_bindgen::to_value(&game).unwrap()
}

#[wasm_bindgen]
pub fn get_allowed_actions(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    serde_wasm_bindgen::to_value(&game.allowed_actions()).unwrap()
}

#[wasm_bindgen]
pub fn get_optimal_move(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    let optimal_move = game.get_optimal_move();
    serde_wasm_bindgen::to_value(&optimal_move).unwrap()
}

#[wasm_bindgen]
pub fn get_player_hand_value(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    let player_hand = &game.player_hands[game.hand_index];
    let aces_split = game.player_split_aces(&game.player_hands);
    let player_hand_value = game.player_hand_value(player_hand, aces_split);
    serde_wasm_bindgen::to_value(&player_hand_value).unwrap()
}

#[wasm_bindgen]
pub fn get_dealer_hand_value(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    let dealer_hand_value = game.dealer_hand_value(&game.dealer_hand, false);
    serde_wasm_bindgen::to_value(&dealer_hand_value).unwrap()
}

#[wasm_bindgen]
pub fn get_game_outcome(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    let player_hand_outcomes = game.player_hand_outcomes();
    serde_wasm_bindgen::to_value(&player_hand_outcomes).unwrap()
}

#[wasm_bindgen]
pub fn monte_carlo(rules: JsValue, iterations: u32) -> () {
    let rules: BlackjackRuleset = serde_wasm_bindgen::from_value(rules).unwrap();
    for _ in 0..iterations {
        let mut game = crate::blackjack::init_state(1f32, rules);
        while !matches!(game.state, GameState::GameOver) {
            if matches!(game.state, GameState::PlayerTurn) {
                let player_action = game.get_optimal_move();
                game.next_state(Some(player_action))
            } else {
                game.next_state(None)
            }
        }
    }
}

fn hand_value(hand: &Vec<&Card>) -> HandValue {
    if hand.len() == 2 {
        let card1 = &hand[0].rank;
        let card2 = &hand[1].rank;
        let is_blackjack = match (card1, card2) {
            (Rank::Ace, Rank::Ten) => true,
            (Rank::Ace, Rank::Jack) => true,
            (Rank::Ace, Rank::Queen) => true,
            (Rank::Ace, Rank::King) => true,
            (Rank::Ten, Rank::Ace) => true,
            (Rank::Jack, Rank::Ace) => true,
            (Rank::Queen, Rank::Ace) => true,
            (Rank::King, Rank::Ace) => true,
            _ => false,
        };
        if is_blackjack {
            return HandValue::Blackjack;
        }
    }
    let has_ace = hand.iter().any(|c| matches!(c.rank, Rank::Ace));
    let low_val: u8 = hand.iter().map(|c| card_value(c, false)).sum();
    match has_ace && low_val <= 11 {
        true => HandValue::Soft(low_val + 10),
        false => HandValue::Hard(low_val),
    }
}

fn _monte_carlo_dealer_only(upcard: Card, iterations: u32) -> HashMap<u8, u32> {
    let mut rng = rand::thread_rng();
    let mut results: HashMap<u8, u32> = HashMap::new();

    for _ in 0..iterations {
        let mut dealer_hand: Vec<&Card> = Vec::with_capacity(4);
        dealer_hand.push(&upcard);
        while {
            let dealer_hand_value = match hand_value(&dealer_hand) {
                HandValue::Soft(value) => value,
                HandValue::Hard(value) => value,
                HandValue::Blackjack => 21,
            };
            dealer_hand_value < 17
        } {
            let random_card: &Card = &UNSHUFFLED_DECK.choose(&mut rng).unwrap();
            dealer_hand.push(random_card);
        }

        let dealer_hand_value = hand_value(&dealer_hand);
        let key = match dealer_hand_value {
            HandValue::Soft(value) => value,
            HandValue::Hard(value) => value,
            HandValue::Blackjack => 21,
        };
        *results.entry(key).or_insert(0) += 1;
    }
    results
}

#[wasm_bindgen]
pub fn monte_carlo_dealer_only(upcard: JsValue, iterations: u32) -> JsValue {
    let upcard: Card = serde_wasm_bindgen::from_value(upcard).unwrap();
    let results = _monte_carlo_dealer_only(upcard, iterations);
    serde_wasm_bindgen::to_value(&results).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::blackjack::Card;

    use super::_monte_carlo_dealer_only;
    #[test]
    fn test_monte_carlo_dealer_only() {
        let results = _monte_carlo_dealer_only(
            Card {
                rank: crate::blackjack::Rank::Two,
                suit: crate::blackjack::Suit::Clubs,
                face_down: false,
            },
            1_000_000,
        );
        dbg!(results);
    }
}

#[wasm_bindgen]
pub fn install_debugging_hook() -> () {
    crate::debugging::set_panic_hook();
}

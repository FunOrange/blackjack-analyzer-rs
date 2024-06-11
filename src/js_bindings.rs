use std::collections::HashMap;

use rand::Rng;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::blackjack::{ruleset::*, BlackjackState, GameState, PlayerAction};

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

fn hand_value(hand: u8, has_ace: bool) -> u8 {
    if has_ace && hand + 10 <= 21 {
        hand + 10
    } else {
        hand
    }
}

const DECK: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10];
const RNG_ARRAY_SIZE: usize = 1000;

pub fn _monte_carlo_dealer_only(upcard: u8, iterations: u32) -> HashMap<u8, u32> {
    let mut rng_array: [u8; RNG_ARRAY_SIZE] = [0; RNG_ARRAY_SIZE];
    let mut rng = rand::thread_rng();

    let mut results: HashMap<u8, u32> = HashMap::new();
    let mut i: usize = 0;
    for _ in 0..iterations {
        // initialize hand and shoe
        let mut dealer_hand = upcard;
        let mut has_ace = upcard == 1;
        let mut dealer_hand_value = 0;
        while dealer_hand_value < 17 {
            let random_byte = {
                let index = i % RNG_ARRAY_SIZE;
                if index == 0 {
                    rng.fill(&mut rng_array[..]);
                }
                i += 1;
                rng_array[index] as usize
            };
            let random_card = DECK[random_byte % 13];
            if random_card == 1 {
                has_ace = true;
            }
            dealer_hand += random_card;
            dealer_hand_value = hand_value(dealer_hand, has_ace);
        }
        *results.entry(dealer_hand_value).or_insert(0) += 1;
    }
    results
}

#[wasm_bindgen]
pub fn monte_carlo_dealer_only(upcard: u8, iterations: u32) -> JsValue {
    let results = _monte_carlo_dealer_only(upcard, iterations);
    serde_wasm_bindgen::to_value(&results).unwrap()
}

#[cfg(test)]
mod tests {
    use super::_monte_carlo_dealer_only;
    #[test]
    fn test_monte_carlo_dealer_only() {
        let start_time = std::time::Instant::now();
        let results = _monte_carlo_dealer_only(6, 500_000);
        let end_time = std::time::Instant::now();
        let duration = end_time - start_time;
        dbg!(results);
        println!("Duration: {:?}", duration);
    }
}

#[wasm_bindgen]
pub fn install_debugging_hook() -> () {
    crate::debugging::set_panic_hook();
}

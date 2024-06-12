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

#[wasm_bindgen]
pub fn simulate_dealer_stand_outcome(upcard: u8, iterations: u32) -> JsValue {
    let results = crate::monte_carlo::simulate_dealer_stand_outcome(upcard, iterations);
    serde_wasm_bindgen::to_value(&results).unwrap()
}

#[wasm_bindgen]
pub fn install_debugging_hook() -> () {
    crate::debugging::set_panic_hook();
}

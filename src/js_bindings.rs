use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::blackjack::{ruleset::*, BlackjackState, PlayerAction};

#[wasm_bindgen]
pub fn create_ruleset(
    surrender: bool,
    dealer_stands_on_all_17: bool,
    dealer_peeks: bool,
    split_aces: JsValue,
    hit_on_split_ace: bool,
    max_hands_after_split: JsValue,
    double_down_on: JsValue,
    double_after_split: bool,
    double_on_split_ace: bool,
    blackjack_payout: f32,
    ace_and_ten_counts_as_blackjack: bool,
    split_ace_can_be_blackjack: bool,
) -> JsValue {
    let split_aces: SplitAces = serde_wasm_bindgen::from_value(split_aces).unwrap();
    let max_hands_after_split: MaxHandsAfterSplit =
        serde_wasm_bindgen::from_value(max_hands_after_split).unwrap();
    let double_down_on: DoubleDownOn = serde_wasm_bindgen::from_value(double_down_on).unwrap();
    let rules = BlackjackRuleset {
        surrender,
        dealer_stands_on_all_17,
        dealer_peeks,
        split_aces,
        hit_on_split_ace,
        max_hands_after_split,
        double_down_on,
        double_after_split,
        double_on_split_ace,
        blackjack_payout,
        ace_and_ten_counts_as_blackjack,
        split_ace_can_be_blackjack,
    };
    serde_wasm_bindgen::to_value(&rules).unwrap()
}

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
pub fn allowed_actions(game: JsValue) -> JsValue {
    let game: BlackjackState = serde_wasm_bindgen::from_value(game).unwrap();
    serde_wasm_bindgen::to_value(&game.allowed_actions()).unwrap()
}

use futures_channel::oneshot;
use js_sys::{Promise, Uint8ClampedArray, WebAssembly};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rayon::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::{
    blackjack::{
        card_value, constants::UNSHUFFLED_DECK, ruleset::*, BlackjackState, Card, GameState,
        HandValue, PlayerAction, Rank,
    },
    pool,
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

fn hand_value(hand: &Vec<u8>) -> u8 {
    let mut hand_value = 0;
    let mut has_ace = false;
    for card_value in hand.iter() {
        if *card_value == 1 {
            has_ace = true;
        }
        hand_value += card_value;
    }
    if has_ace && hand_value + 10 <= 21 {
        hand_value += 10;
    }
    hand_value
}
const DECK: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10];

pub fn _monte_carlo_dealer_only(upcard: u8, iterations: u32) -> HashMap<u8, u32> {
    let mut rng = rand::thread_rng();
    let mut results: HashMap<u8, u32> = HashMap::new();

    let mut dealer_hand: Vec<u8> = Vec::with_capacity(21);
    for _ in 0..iterations {
        // initialize hand and shoe
        dealer_hand.clear();
        dealer_hand.push(upcard);
        while hand_value(&dealer_hand) < 17 {
            let i = rng.gen_range(0..DECK.len());
            let random_card = DECK[i];
            dealer_hand.push(random_card);
        }
        let dealer_hand_value = hand_value(&dealer_hand);
        *results.entry(dealer_hand_value).or_insert(0) += 1;
    }
    results
}

#[wasm_bindgen]
pub fn monte_carlo_dealer_only(
    upcard: u8,
    iterations: u32,
    concurrency: usize,
    pool: &pool::WorkerPool,
) -> Result<Promise, JsValue> {
    let mut results: Vec<HashMap<u8, u32>> = Vec::new();
    for _ in 0..concurrency {
        results.push(HashMap::new());
    }

    // Configure a rayon thread pool which will pull web workers from
    // `pool`.
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(concurrency)
        .spawn_handler(|thread| {
            pool.run(|| thread.run()).unwrap();
            Ok(())
        })
        .build()
        .unwrap();

    // And now execute the render! The entire render happens on our worker
    // threads so we don't lock up the main thread, so we ship off a thread
    // which actually does the whole rayon business. When our returned
    // future is resolved we can pull out the final version of the image.
    let (tx, rx) = oneshot::channel();
    pool.run(move || {
        thread_pool.install(|| {
            results
                .par_chunks_mut(1)
                .enumerate()
                .for_each(|(_, chunk)| {
                    let results = _monte_carlo_dealer_only(upcard, iterations / concurrency as u32);
                    chunk[0] = results;
                });
        });
        drop(tx.send(results));
    })?;

    let done = async move {
        match rx.await {
            Ok(results) => Ok(serde_wasm_bindgen::to_value(&results).unwrap()),
            Err(_) => Err(JsValue::undefined()),
        }
    };
    Ok(wasm_bindgen_futures::future_to_promise(done))
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

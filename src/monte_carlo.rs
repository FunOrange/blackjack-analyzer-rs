use rand::Rng;
use std::collections::HashMap;

const DECK: [u8; 13] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10];
const RNG_ARRAY_SIZE: usize = 1000;

fn hand_value(hand: u8, has_ace: bool) -> u8 {
    if has_ace && hand + 10 <= 21 {
        hand + 10
    } else {
        hand
    }
}
pub fn simulate_dealer_stand_outcome(upcard: u8, iterations: u32) -> HashMap<u8, u32> {
    let mut rng_array: [u8; RNG_ARRAY_SIZE] = [0; RNG_ARRAY_SIZE];
    let mut rng = rand::thread_rng();

    let mut results: HashMap<u8, u32> = HashMap::new();
    let mut i: usize = 0;
    for _ in 0..iterations {
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

#[cfg(test)]
mod tests {
    use crate::simulate_dealer_stand_outcome;

    #[test]
    fn test_simulate_dealer_stand_outcome() {
        let iterations = 100_000;
        let start_time = std::time::Instant::now();
        let results = simulate_dealer_stand_outcome(6, iterations);
        let end_time = std::time::Instant::now();
        let duration = end_time - start_time;
        dbg!(results);
        println!("Ran {:?} simulations in {:?}", iterations, duration);
    }
}

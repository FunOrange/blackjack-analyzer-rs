mod blackjack;
mod constants;
use blackjack::{
    init_state, next_state,
    ruleset::{BlackjackRuleset, DoubleDownOn, MaxHandsAfterSplit, SplitAces},
};

fn main() {
    let rules = BlackjackRuleset {
        dealer_stands_on_all_17: true,
        dealer_peeks: true,

        split_aces: SplitAces::Thrice,
        hit_on_split_ace: false,
        max_hands_after_split: MaxHandsAfterSplit::Four,

        double_down_on: DoubleDownOn::Any,
        double_after_split: true,
        double_on_split_ace: false,

        ace_and_ten_counts_as_blackjack: true,
        blackjack_payout: 3.0 / 2.0,
        split_ace_can_be_blackjack: false,
    };
    let starting_bet = 1f32;

    let mut game = init_state(starting_bet, rules);
    for _ in 0..4 {
        game = next_state(game, None);
        dbg!(&game);
    }
}

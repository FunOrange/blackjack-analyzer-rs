use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum SplitAces {
    NotAllowed,
    Once,
    Twice,
    Thrice,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MaxHandsAfterSplit {
    One,
    Two,
    Three,
    Four,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DoubleDownOn {
    Any,
    NineTenEleven,
    TenEleven,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BlackjackRuleset {
    pub surrender: bool,

    // dealer
    pub dealer_stands_on_all_17: bool,
    pub dealer_peeks: bool,

    // splitting
    pub split_aces: SplitAces,
    pub hit_on_split_ace: bool,
    pub max_hands_after_split: MaxHandsAfterSplit,

    // doubling
    pub double_down_on: DoubleDownOn,
    pub double_after_split: bool,
    pub double_on_split_ace: bool,

    // blackjack
    pub blackjack_payout: f32,
    pub ace_and_ten_counts_as_blackjack: bool,
    pub split_ace_can_be_blackjack: bool,
}

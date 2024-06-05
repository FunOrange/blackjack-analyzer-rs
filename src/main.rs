mod blackjack;
mod constants;
mod terminal;
use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use blackjack::{
    init_state,
    ruleset::{BlackjackRuleset, DoubleDownOn, MaxHandsAfterSplit, SplitAces},
    HandOutcome, LossReason, PlayerAction, WinReason,
};
use terminal::{clear_screen, green, red, yellow};

enum TitleScreenInput {
    PlayGame,
    MonteCarloSimulation,
}
fn get_title_screen_input() -> TitleScreenInput {
    print!("Please enter a number between 1 and 3: ");
    let _ = io::stdout().flush(); // Make sure the prompt is immediately displayed

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    match input.trim().parse::<i32>() {
        Ok(1) => TitleScreenInput::PlayGame,
        Ok(3) => TitleScreenInput::MonteCarloSimulation,
        _ => {
            println!("Invalid input. Please try again.");
            get_title_screen_input()
        }
    }
}

fn get_player_input(allowed_actions: &Vec<PlayerAction>) -> blackjack::PlayerAction {
    for (i, action) in allowed_actions.iter().enumerate() {
        println!(
            "{}: {}",
            i + 1,
            match action {
                PlayerAction::Hit => "Hit",
                PlayerAction::Stand => "Stand",
                PlayerAction::DoubleDown => "Double Down",
                PlayerAction::Split => "Split",
            }
        );
    }
    print!("Please enter your move: ");
    let _ = io::stdout().flush(); // Make sure the prompt is immediately displayed
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    match input.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= allowed_actions.len() => allowed_actions[n - 1],
        _ => {
            println!("Invalid input. Please try again.");
            get_player_input(allowed_actions)
        }
    }
}

const RULES: BlackjackRuleset = BlackjackRuleset {
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

fn main() {
    println!("Welcome to Blackjack!");
    println!("1: Play game");
    println!("2: Monte Carlo Simulation");
    match get_title_screen_input() {
        TitleScreenInput::PlayGame => manual_play(),
        TitleScreenInput::MonteCarloSimulation => monte_carlo_simulation(),
    }
}

fn manual_play() {
    let starting_bet = 1f32;
    let mut bankroll = 1000f32;
    let flat_bet = 1f32;
    loop {
        let starting_balance = bankroll;
        bankroll -= flat_bet;
        let mut game = init_state(starting_bet, RULES);

        while !matches!(game.state, blackjack::GameState::GameOver) {
            clear_screen();
            game.print_game_state();
            game = match game.state {
                blackjack::GameState::Dealing | blackjack::GameState::DealerTurn => {
                    thread::sleep(Duration::from_millis(150));
                    game.next_state(None)
                }
                blackjack::GameState::PlayerTurn => {
                    let allowed_actions = game.allowed_actions();
                    let player_action = get_player_input(&allowed_actions);
                    if matches!(
                        player_action,
                        PlayerAction::DoubleDown | PlayerAction::Split
                    ) {
                        bankroll -= flat_bet;
                    }
                    game.next_state(Some(player_action))
                }
                blackjack::GameState::GameOver => panic!("Unreachable code."),
            }
        }
        clear_screen();
        game.print_game_state();
        let earnings = {
            let player_hand_outcomes = game.player_hand_outcomes();
            let mut earnings = 0f32;
            for (bet, outcome) in game.bets.iter().zip(player_hand_outcomes) {
                earnings += match outcome {
                    HandOutcome::Win(WinReason::Blackjack) => {
                        println!("{}", green("Blackjack!"));
                        game.rules.blackjack_payout * (*bet * 2f32)
                    }
                    HandOutcome::Win(WinReason::DealerBust) => {
                        println!("{}", green("Dealer busts!"));
                        *bet * 2f32
                    }
                    HandOutcome::Win(WinReason::HigherHand) => {
                        println!("{}", green("Player Wins!"));
                        *bet * 2f32
                    }
                    HandOutcome::Push => {
                        println!("{}", yellow("Push."));
                        *bet
                    }
                    HandOutcome::Lose(LossReason::Bust) => {
                        println!("{}", red("Bust."));
                        0f32
                    }
                    HandOutcome::Lose(LossReason::LowerHand) => {
                        println!("{}", red("Dealer wins."));
                        0f32
                    }
                    HandOutcome::Lose(LossReason::DealerBlackjack) => {
                        println!("{}", red("Dealer has blackjack."));
                        0f32
                    }
                }
            }
            earnings
        };
        bankroll += earnings;
        println!(
            "Bankroll: ${:.2} {}",
            bankroll,
            if bankroll > starting_balance {
                green(format!("(+${:.2})", bankroll - starting_balance).as_str())
            } else {
                red(format!("(-${:.2})", starting_balance - bankroll).as_str())
            }
        );
        print!("Press Enter to play again:");
        let _ = io::stdout().flush(); // Make sure the prompt is immediately displayed
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
    }
}
fn monte_carlo_simulation() {}

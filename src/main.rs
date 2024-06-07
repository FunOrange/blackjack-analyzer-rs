mod blackjack;
mod constants;
mod terminal;
use num_format::{Locale, ToFormattedString};
use rand::prelude::SliceRandom;
use std::{
    collections::HashMap,
    io::{self, Write},
    sync::mpsc,
    thread,
    time::{Duration, SystemTime},
};

use blackjack::{
    init_state,
    ruleset::{BlackjackRuleset, DoubleDownOn, MaxHandsAfterSplit, SplitAces},
    GameState, HandOutcome, LossReason, PlayerAction, WinReason,
};
use terminal::{clear_screen, green, red, yellow};

enum TitleScreenInput {
    PlayGame,
    AutoPlay,
    MonteCarloSimulation,
}
fn get_title_screen_input() -> TitleScreenInput {
    print!("Please enter a number between 1 and 3: ");
    let _ = io::stdout().flush(); // Make sure the prompt is immediately displayed

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    match input.trim().parse::<i32>() {
        Ok(1) => TitleScreenInput::PlayGame,
        Ok(2) => TitleScreenInput::AutoPlay,
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
                PlayerAction::Surrender => "Surrender",
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
    surrender: true,

    dealer_stands_on_all_17: true,
    dealer_peeks: true,

    split_aces: SplitAces::Twice,
    hit_on_split_ace: false,
    max_hands_after_split: MaxHandsAfterSplit::Three,

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
    println!("2: Auto play");
    println!("3: Monte Carlo Simulation");
    match get_title_screen_input() {
        TitleScreenInput::PlayGame => play(false),
        TitleScreenInput::AutoPlay => play(true),
        TitleScreenInput::MonteCarloSimulation => monte_carlo_simulation(),
    }
}

const FLAT_BET: f32 = 1f32;
fn play(auto_play: bool) {
    let mut bankroll = 1000f32;
    loop {
        let starting_balance = bankroll;
        bankroll -= FLAT_BET;
        let mut game = init_state(FLAT_BET, RULES);

        while !matches!(game.state, blackjack::GameState::GameOver) {
            clear_screen();
            game.print_game_state();
            match game.state {
                blackjack::GameState::Dealing | blackjack::GameState::DealerTurn => {
                    thread::sleep(Duration::from_millis(150));
                    game.next_state(None);
                }
                blackjack::GameState::PlayerTurn => {
                    let allowed_actions = game.allowed_actions();
                    let player_action = match auto_play {
                        true => game.get_optimal_move(),
                        false => get_player_input(&allowed_actions),
                    };
                    if matches!(
                        player_action,
                        PlayerAction::DoubleDown | PlayerAction::Split
                    ) {
                        bankroll -= FLAT_BET;
                    }
                    game.next_state(Some(player_action));
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
                    HandOutcome::Won(WinReason::Blackjack) => {
                        println!("{}", green("Blackjack!"));
                        game.rules.blackjack_payout * (*bet * 2f32)
                    }
                    HandOutcome::Won(WinReason::DealerBust) => {
                        println!("{}", green("Dealer busts!"));
                        *bet * 2f32
                    }
                    HandOutcome::Won(WinReason::HigherHand) => {
                        println!("{}", green("Player Wins!"));
                        *bet * 2f32
                    }
                    HandOutcome::Push => {
                        println!("{}", yellow("Push."));
                        *bet
                    }
                    HandOutcome::Lost(LossReason::Bust) => {
                        println!("{}", red("Bust."));
                        0f32
                    }
                    HandOutcome::Lost(LossReason::LowerHand) => {
                        println!("{}", red("Dealer wins."));
                        0f32
                    }
                    HandOutcome::Lost(LossReason::DealerBlackjack) => {
                        println!("{}", red("Dealer has blackjack."));
                        0f32
                    }
                    HandOutcome::Surrendered => {
                        println!("{}", yellow("Surrendered."));
                        *bet / 2f32
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

const NUM_THREADS: usize = 16;
const TX_INTERVAL: Duration = Duration::from_millis(800);
fn monte_carlo_simulation() {
    let (tx, rx) = mpsc::channel();

    for i in 0..16 {
        let thread_tx = tx.clone();
        thread::spawn(move || {
            let mut send_time = SystemTime::now();
            // staggered start times
            thread::sleep(Duration::from_millis(
                (i as f32 * ((TX_INTERVAL.as_millis() as f32) / (NUM_THREADS as f32))) as u64,
            ));
            loop {
                let mut net_earnings_distribution: HashMap<i32, u32> = HashMap::new();
                let mut i = 1;
                loop {
                    let mut bankroll = 0f32;
                    // let preround_bankroll = 0;
                    bankroll -= FLAT_BET;
                    let mut game = init_state(FLAT_BET, RULES);

                    while !matches!(game.state, GameState::GameOver) {
                        if matches!(game.state, GameState::PlayerTurn) {
                            let allowed_actions = game.allowed_actions();
                            let mut rng = rand::thread_rng();
                            let random_action = allowed_actions.choose(&mut rng).unwrap();
                            if matches!(
                                random_action,
                                PlayerAction::DoubleDown | PlayerAction::Split
                            ) {
                                bankroll -= FLAT_BET;
                            }
                            game.next_state(Some(*random_action))
                        } else {
                            game.next_state(None)
                        }
                    }
                    let player_hand_outcomes = game.player_hand_outcomes();
                    for (bet, outcome) in game.bets.iter().zip(player_hand_outcomes) {
                        bankroll += match outcome {
                            HandOutcome::Won(WinReason::Blackjack) => {
                                *bet + game.rules.blackjack_payout * *bet
                            }
                            HandOutcome::Won(_) => *bet + *bet,
                            HandOutcome::Push => *bet,
                            _ => 0f32,
                        }
                    }
                    let net = bankroll;
                    let net_cents = (net * 100f32).round() as i32;
                    let zero: u32 = 0;
                    net_earnings_distribution.insert(
                        net_cents,
                        net_earnings_distribution.get(&net_cents).unwrap_or(&zero) + 1,
                    );
                    if SystemTime::now()
                        .duration_since(send_time)
                        .unwrap_or(Duration::from_millis(1))
                        < TX_INTERVAL
                    {
                        thread_tx
                            .send((net_earnings_distribution.clone(), i))
                            .unwrap();
                        send_time = SystemTime::now();
                        i = 1;
                        net_earnings_distribution.clear();
                    }
                    i += 1;
                }
            }
        });
    }

    let start_time = SystemTime::now();
    let mut net_earnings_distribution: HashMap<i32, u32> = HashMap::new();
    let mut iterations = 1;
    let mut last_print_time = SystemTime::now();
    // let mut j = 0;
    loop {
        let (net_earnings_distribution2, i) = rx.recv().unwrap();
        iterations += i;
        for (key, value) in net_earnings_distribution2 {
            net_earnings_distribution = {
                let mut map = net_earnings_distribution.clone();
                *map.entry(key).or_insert(0) += value;
                map
            }
        }
        const DRAW_INTERVAL: Duration = Duration::from_millis(1000 / 160);
        if SystemTime::now()
            .duration_since(last_print_time)
            .unwrap_or(Duration::from_millis(1))
            > DRAW_INTERVAL
        {
            clear_screen();
            print_stats(&start_time, &iterations, &net_earnings_distribution);
            last_print_time = SystemTime::now();
            if SystemTime::now()
                .duration_since(start_time)
                .unwrap_or(Duration::from_millis(1))
                > Duration::from_secs(5)
            {
                std::process::exit(0);
            }
        }
    }
}

fn print_stats(
    start_time: &SystemTime,
    iterations: &u32,
    net_earnings_distribution: &HashMap<i32, u32>,
) {
    // println!("Starting bankroll: ${}", *initial_bankroll);
    // let net = *bankroll - *initial_bankroll;
    // {
    //     print!("Bankroll: ${}", bankroll);
    //     println!(
    //         " {}",
    //         if net > 0f32 {
    //             green(format!("+${}", net).as_str())
    //         } else if net < 0f32 {
    //             red(format!("-${}", net.abs()).as_str())
    //         } else {
    //             "".to_string()
    //         }
    //     );
    // };
    println!("Loss/earnings distribution:");
    let mut vec = net_earnings_distribution.iter().collect::<Vec<_>>();
    vec.sort_by(|a, b| a.0.cmp(&b.0));
    for (cents, count) in vec {
        let percent = (*count as f32 / *iterations as f32) * 100f32;
        let count = (*count).to_formatted_string(&Locale::en);
        let dollars = (*cents as f32 / 100f32).abs();
        if *cents > 0 {
            println!(
                "{}: {:.2}% ({})",
                green(format!("+${:.2}", dollars).as_str()),
                percent,
                count
            )
        } else if *cents < 0 {
            println!(
                "{}: {:.2}% ({})",
                red(format!("-${:.2}", dollars).as_str()),
                percent,
                count
            )
        } else if *cents == 0 {
            println!("$0: {:.2}% ({})", percent, count)
        }
    }
    // let house_edge_percent = ((*initial_bankroll - *bankroll) / *i as f32 / FLAT_BET) * 100f32;
    // println!("House edge: {:.2}%", house_edge_percent);
    let duration = SystemTime::now()
        .duration_since(*start_time)
        .unwrap_or(Duration::from_millis(1));
    println!(
        "Simulated {} rounds in {:.2} seconds",
        (*iterations).to_formatted_string(&Locale::en).as_str(),
        duration.as_millis() as f32 / 1000f32
    );
}

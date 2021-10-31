use num_bigint::BigUint;
use num_traits::{CheckedSub, One};
use sixtyfps::SharedString;

use getset::{Getters, MutGetters, Setters};

use std::convert::TryInto;
use std::sync::{Arc, Mutex};

sixtyfps::include_modules!();

type ArcState = Arc<Mutex<State>>;

const BASE_COST: usize = 1000;
const INIT_NUMBER: &[u8] = b"1";
const NOTATION_AMOUNT: usize = 6;
const NOTATION_PRECISION: usize = 5;

#[derive(Debug, PartialEq, Getters, Setters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub")]
pub struct State {
    base: BigUint,
    counter: BigUint,
    multiplier: BigUint,
}

impl State {
    pub fn update_counter(&mut self) {
        let new_counter = (self.base() * self.counter()) + self.multiplier();
        self.set_counter(new_counter);
    }

    pub fn update_multiplier(&mut self) -> bool {
        if let Some(new_counter) = self.counter().checked_sub(self.multiplier()) {
            self.set_counter(new_counter);
            self.set_multiplier(self.multiplier() + 1u64);
            true
        } else {
            false
        }
    }

    pub fn update_base(&mut self) -> bool {
        let base_collection = self.base().iter_u32_digits().collect::<Vec<u32>>();
        let power = if base_collection.len() == 1 {
            base_collection[0]
        } else {
            u32::MAX
        };
        let base_cost = BigUint::from(BASE_COST).pow(power);

        if let Some(new_counter) = self.counter().checked_sub(&base_cost) {
            self.set_counter(new_counter);
            self.set_base(self.base() + 1u64);
            true
        } else {
            false
        }
    }
}

impl Default for State {
    fn default() -> State {
        State {
            base: BigUint::one(),
            counter: BigUint::parse_bytes(INIT_NUMBER, 10).unwrap(),
            multiplier: BigUint::one(),
        }
    }
}

fn main() {
    let ui = AppWindow::new();
    ui.set_a_width(640.0);
    ui.set_a_height(640.0);

    let state = Arc::new(Mutex::new(State::default()));

    let ui_handle = ui.as_weak();

    let ui_handle_clone = ui_handle.clone();
    let state_clone = state.clone();
    ui.on_request_increase_value(move || {
        increase_counter(ui_handle_clone.clone(), state_clone.clone())
    });

    let ui_handle_clone = ui_handle.clone();
    let state_clone = state.clone();
    ui.on_request_increase_multiplier(move || {
        increase_multiplier(ui_handle_clone.clone(), state_clone.clone())
    });

    let ui_handle_clone = ui_handle.clone();
    let state_clone = state.clone();
    ui.on_request_increase_base(move || {
        increase_base(ui_handle_clone.clone(), state_clone.clone())
    });

    ui.run();
}

fn increase_counter(
    ui_handle: sixtyfps::Weak<sixtyfps_generated_AppWindow::AppWindow>,
    state: ArcState,
) {
    let ui = ui_handle.unwrap();

    let mut state = state.lock().unwrap();

    state.update_counter();
    ui.set_counter(print_number(state.counter()));
}

fn increase_multiplier(
    ui_handle: sixtyfps::Weak<sixtyfps_generated_AppWindow::AppWindow>,
    state: ArcState,
) {
    let ui = ui_handle.unwrap();

    let mut state = state.lock().unwrap();

    if state.update_multiplier() {
        ui.set_counter(print_number(state.counter()));
        ui.set_multiplier(print_number(state.multiplier()));
    }
}

fn increase_base(
    ui_handle: sixtyfps::Weak<sixtyfps_generated_AppWindow::AppWindow>,
    state: ArcState,
) {
    let ui = ui_handle.unwrap();

    let mut state = state.lock().unwrap();

    if state.update_base() {
        ui.set_counter(print_number(state.counter()));
        ui.set_base(print_number(state.base()));
    }
}

fn print_number(counter: &BigUint) -> SharedString {
    // SharedString::from(textwrap::wrap(&counter.to_string(), 32).join("\n"))
    SharedString::from(to_notation(counter))
}

fn to_notation(int: &BigUint) -> String {
    let txt = int.to_string();

    match divmod(txt.len(), NOTATION_AMOUNT) {
        (0, 0) => unreachable!(),
        (0, _) => txt,
        (div, mut rem) => {
            let number: f32 = txt
                .get(0..NOTATION_PRECISION)
                .unwrap_or(&txt)
                .parse()
                .unwrap();
            rem = NOTATION_PRECISION - rem;
            let power = 10usize.pow(rem.try_into().expect("0..6 always fits into u32")) as f32;
            format!("{}e{}", number / power, div * NOTATION_AMOUNT)
        }
    }
}

fn divmod(integer: usize, rem: usize) -> (usize, usize) {
    (integer / rem, integer % rem)
}

#[test]
fn notation_tests() {
    let tests: Vec<(&str, &[u8])> = vec![
        ("1234", b"1234"),
        ("12345", b"12345"),
        ("0.12345e6", b"123456"),
        ("1.2345e6", b"1234567"),
        ("12.345e6", b"12345678"),
        ("123.45e6", b"123456789"),
        ("1234.5e6", b"1234567890"),
        ("12345e6", b"12345678901"),
        ("0.12345e12", b"123456789012"),
        ("1.2345e12", b"1234567890123"),
        ("0.12345e30", b"123456789012345678901234567890"),
        (
            "1234.5e60",
            b"1234567890123456789012345678901234567890123456789012345678901234",
        ),
    ];

    for (expected, input) in tests {
        assert_eq!(
            expected,
            to_notation(&BigUint::parse_bytes(input, 10).unwrap())
        )
    }
}

#[test]
fn state_update_counter() {
    let mut state = State::default();

    state.update_counter();
    assert_eq!(&BigUint::from(2usize), state.counter());
    state.update_counter();
    assert_eq!(&BigUint::from(3usize), state.counter());
}

#[test]
fn state_update_multiplier_success() {
    let mut state = State::default();
    let a = state.counter_mut();
    *a += 123456789usize;

    state.update_multiplier();

    assert_eq!(&BigUint::from(123456789usize), state.counter());
    assert_eq!(&BigUint::from(2usize), state.multiplier());
}

#[test]
fn state_update_multiplier_invalid() {
    let mut state = State::default();

    state.update_multiplier();

    assert_eq!(&BigUint::from(0usize), state.counter());
    assert_eq!(&BigUint::from(2usize), state.multiplier());

    // wont update
    state.update_multiplier();

    assert_eq!(&BigUint::from(0usize), state.counter());
    assert_eq!(&BigUint::from(2usize), state.multiplier());
}

#[test]
fn state_update_base_success() {
    let mut state = State::default();
    let a = state.counter_mut();
    *a += 1000usize;

    state.update_base();

    assert_eq!(&BigUint::from(1usize), state.counter());
    assert_eq!(&BigUint::from(2usize), state.base());
}

#[test]
fn state_update_base_invalid() {
    let mut state = State::default();
    let a = state.counter_mut();
    *a += 850usize;

    state.update_base();

    assert_eq!(&BigUint::from(851usize), state.counter());
    assert_eq!(&BigUint::from(1usize), state.base());
}

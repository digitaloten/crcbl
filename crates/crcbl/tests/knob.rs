//! [`crcbl::knob::Knob`], driven against console variables of this test's own.
//!
//! **An integration test rather than a unit one**, and that is the point of the
//! fixtures below: `crcbl_console::guard::declared_names` holds
//! [`crcbl::console_table`] to every `convar!` under `crates/crcbl/src`, so a
//! variable declared beside the code it exercises would be one the engine's own
//! table is then required to publish. Here it is out of that scan, and the knob
//! is driven through the public API a sample uses.

use crcbl::console::{Table, Value};
use crcbl::knob::{Knob, reset_all};

crcbl_console::convar! {
    /// A float with a range of its own.
    pub static test_knob_radius: f32 in 0.25..=4.0 = 1.0;
}
crcbl_console::convar! {
    /// A name out of a set.
    pub static test_knob_technique: &'static str one_of ["gtao", "hbao", "ssao"] = "gtao";
}
crcbl_console::convar! {
    /// A switch.
    pub static test_knob_bent: bool = false;
}

/// Serialises every check that moves a knob, and puts them all back.
///
/// A `ConVar` is process-global by design and `cargo test` runs a crate's
/// tests as threads of one process, so two checks that move the radius are
/// two writers to one cell — which shows up as a flake rather than as a
/// failure anybody can read.
static KNOB_SWITCH: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Held {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for Held {
    fn drop(&mut self) {
        reset_all(table(), &NAMES);
    }
}

const NAMES: [&str; 3] = ["test_knob_radius", "test_knob_technique", "test_knob_bent"];

fn table() -> Table {
    crcbl_console::table![test_knob_radius, test_knob_technique, test_knob_bent]
}

fn held() -> Held {
    let guard = KNOB_SWITCH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_all(table(), &NAMES);
    Held { _guard: guard }
}

/// **Cycling walks the variable's own set and comes back round**, rather
/// than a set written down beside the caller.
#[test]
fn cycling_reaches_every_declared_name_and_wraps() {
    let _held = held();
    let knob = Knob::named(table(), "test_knob_technique");
    let declared = knob.names();
    assert!(declared.len() > 1, "a set of one cycles nowhere");

    let start = knob.var().get_enum();
    let mut seen = vec![start];
    for _ in 1..declared.len() {
        knob.cycle();
        let now = knob.var().get_enum();
        assert!(!seen.contains(&now), "cycling repeated {now} early");
        seen.push(now);
    }
    knob.cycle();
    assert_eq!(knob.var().get_enum(), start, "the cycle must wrap");
    seen.sort_unstable();
    let mut want = declared.to_vec();
    want.sort_unstable();
    assert_eq!(seen, want, "cycling must reach every declared name");
}

/// **A float stops at the variable's own bound**, at both ends, rather than
/// at one the caller wrote down — and the clamp is what keeps a value from
/// outside the range from being refused outright and moving nothing.
#[test]
fn a_float_is_clamped_into_the_range_the_variable_declares() {
    let _held = held();
    let knob = Knob::named(table(), "test_knob_radius");
    let (min, max) = knob.range().expect("a float variable");
    assert!((knob.set_float(max * 100.0) - max).abs() < 1e-6);
    assert!((knob.var().get_f32() - max).abs() < 1e-6);
    assert!((knob.set_float(min / 100.0) - min).abs() < 1e-6);
    assert!((knob.var().get_f32() - min).abs() < 1e-6);
}

/// **A knob that is not a float declares no range and no name set**, which
/// is how a caller that does not know what kind it holds asks.
#[test]
fn a_knob_answers_for_the_kind_it_actually_holds() {
    let _held = held();
    let switch = Knob::named(table(), "test_knob_bent");
    assert_eq!(switch.range(), None);
    assert!(switch.names().is_empty(), "a bool declares no name set");
    switch.cycle();
    assert!(!switch.var().get_bool(), "cycling moved a switch");
    assert_eq!(
        Knob::named(table(), "test_knob_radius").names(),
        &[] as &[&str]
    );
}

/// **Writing a float into a switch is a mistake in the caller's names**,
/// and it says so rather than moving nothing quietly.
#[test]
#[should_panic(expected = "`test_knob_bent` is not a float")]
fn set_float_on_a_switch_names_the_variable_it_refused() {
    let _held = held();
    Knob::named(table(), "test_knob_bent").set_float(1.0);
}

/// **`reset_all` puts every named knob back**, which is what a run that has
/// been experimented on owes the next frame it means to compare.
#[test]
fn reset_all_puts_every_named_knob_back_to_its_declared_default() {
    let _held = held();
    let table = table();
    let before: Vec<Value> = NAMES
        .iter()
        .map(|name| Knob::named(table, name).var().get())
        .collect();
    Knob::named(table, "test_knob_technique").cycle();
    Knob::named(table, "test_knob_radius").set_float(3.5);
    Knob::named(table, "test_knob_bent").set(&Value::Bool(true));
    let moved: Vec<Value> = NAMES
        .iter()
        .map(|name| Knob::named(table, name).var().get())
        .collect();
    assert_ne!(before, moved, "nothing moved, so there is nothing to reset");

    reset_all(table, &NAMES);
    for name in NAMES {
        let knob = Knob::named(table, name);
        assert_eq!(&knob.var().get(), knob.var().default(), "{name}");
    }
}

/// **A name the table does not declare is a mistake in the caller's list**,
/// and it says which name rather than answering with a knob wired to
/// nothing.
#[test]
#[should_panic(expected = "declares no `r_not_a_variable` variable")]
fn a_name_the_table_does_not_declare_panics_with_the_name() {
    let _ = Knob::named(table(), "r_not_a_variable");
}

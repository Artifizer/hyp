//! Problem Examples CLI
//!
//! A command-line tool to explore and demonstrate Rust code problems.

use clap::{Parser, Subcommand};

// Direct imports for all problem entry functions
use problem_examples::e10_unsafe_code::e1001_direct_panic::e1001_entry;
use problem_examples::e10_unsafe_code::e1002_direct_unwrap_expect::e1002_entry;
use problem_examples::e10_unsafe_code::e1003_unsafe_code::e1003_entry;
use problem_examples::e10_unsafe_code::e1004_unsafe_without_comment::e1004_entry;
use problem_examples::e10_unsafe_code::e1005_raw_pointer_deref::e1005_entry;
use problem_examples::e10_unsafe_code::e1006_unsafe_transmute::e1006_entry;
use problem_examples::e10_unsafe_code::e1007_null_pointer_deref::e1007_entry;
use problem_examples::e10_unsafe_code::e1008_unsafe_trait_impl::e1008_entry;
use problem_examples::e10_unsafe_code::e1009_unsafe_cell_misuse::e1009_entry;
use problem_examples::e10_unsafe_code::e1010_mutable_static::e1010_entry;
use problem_examples::e10_unsafe_code::e1011_uninitialized_memory::e1011_entry;
use problem_examples::e10_unsafe_code::e1012_unsafe_auto_trait::e1012_entry;
use problem_examples::e10_unsafe_code::e1013_union_field_access::e1013_entry;
use problem_examples::e10_unsafe_code::e1014_raw_pointer_arithmetic::e1014_entry;
use problem_examples::e10_unsafe_code::e1015_unwrap_expect_wo_context::e1015_entry;
use problem_examples::e10_unsafe_code::e1016_mutex_unwrap::e1016_entry;
use problem_examples::e10_unsafe_code::e1017_todo_unimplemented::e1017_entry;
use problem_examples::e10_unsafe_code::e1018_prohibit_transmute::e1018_entry;
use problem_examples::e11_code_surface_complexity::e1101_high_cyclomatic_complexity::e1101_entry;
use problem_examples::e11_code_surface_complexity::e1102_deep_nested_logic_in_loops::e1102_entry;
use problem_examples::e11_code_surface_complexity::e1103_too_many_params::e1103_entry;
use problem_examples::e11_code_surface_complexity::e1104_overly_large_struct::e1104_entry;
use problem_examples::e11_code_surface_complexity::e1105_boolean_params::e1105_entry;
use problem_examples::e11_code_surface_complexity::e1106_long_function::e1106_entry;
use problem_examples::e11_code_surface_complexity::e1107_deep_nesting::e1107_entry;
use problem_examples::e11_code_surface_complexity::e1108_nested_match::e1108_entry;
use problem_examples::e11_code_surface_complexity::e1109_excessive_chaining::e1109_entry;
use problem_examples::e11_code_surface_complexity::e1110_nested_callbacks::e1110_entry;
use problem_examples::e11_code_surface_complexity::e1111_excessive_tuple_complexity::e1111_entry;
use problem_examples::e11_code_surface_complexity::e1112_magic_numbers::e1112_entry;
use problem_examples::e12_code_pattern_complexity::e1201_complex_generics::e1201_entry;
use problem_examples::e12_code_pattern_complexity::e1202_complex_lifetimes::e1202_entry;
use problem_examples::e12_code_pattern_complexity::e1203_complicated_borrowing::e1203_entry;
use problem_examples::e12_code_pattern_complexity::e1204_trait_method_ambiguity::e1204_entry;
use problem_examples::e12_code_pattern_complexity::e1205_complex_handler::e1205_entry;
use problem_examples::e12_code_pattern_complexity::e1206_deeply_nested_generics::e1206_entry;
use problem_examples::e12_code_pattern_complexity::e1207_complex_user_generics::e1207_entry;
use problem_examples::e12_code_pattern_complexity::e1208_phantom_types::e1208_entry;
use problem_examples::e12_code_pattern_complexity::e1209_hrtb::e1209_entry;
use problem_examples::e12_code_pattern_complexity::e1210_recursive_types::e1210_entry;
use problem_examples::e12_code_pattern_complexity::e1211_trait_object_complexity::e1211_entry;
use problem_examples::e12_code_pattern_complexity::e1212_gat_complexity::e1212_entry;
use problem_examples::e12_code_pattern_complexity::e1213_const_generic_complexity::e1213_entry;
use problem_examples::e12_code_pattern_complexity::e1214_macro_generated::e1214_entry;
use problem_examples::e12_code_pattern_complexity::e1215_type_level_programming::e1215_entry;
use problem_examples::e12_code_pattern_complexity::e1216_chained_transform::e1216_entry;
use problem_examples::e12_code_pattern_complexity::e1217_abba_deadlock::e1217_entry;
use problem_examples::e13_error_handling::e1301_unhandled_result::e1301_entry;
use problem_examples::e13_error_handling::e1302_constructor_without_result::e1302_entry;
use problem_examples::e13_error_handling::e1303_ignored_errors::e1303_entry;
use problem_examples::e13_error_handling::e1304_unwrap_in_error_path::e1304_entry;
use problem_examples::e13_error_handling::e1305_non_exhaustive_match::e1305_entry;
use problem_examples::e13_error_handling::e1306_swallow_errors::e1306_entry;
use problem_examples::e13_error_handling::e1307_string_errors::e1307_entry;
use problem_examples::e13_error_handling::e1308_not_using_question_mark::e1308_entry;
use problem_examples::e13_error_handling::e1309_panic_in_drop::e1309_entry;
use problem_examples::e13_error_handling::e1310_error_context_loss::e1310_entry;
use problem_examples::e14_type_safety::e1401_integer_overflow::e1401_entry;
use problem_examples::e14_type_safety::e1402_division_by_zero::e1402_entry;
use problem_examples::e14_type_safety::e1403_modulo_by_zero::e1403_entry;
use problem_examples::e14_type_safety::e1404_narrowing_conversion::e1404_entry;
use problem_examples::e14_type_safety::e1405_integer_division_rounding::e1405_entry;
use problem_examples::e14_type_safety::e1406_signed_unsigned_mismatch::e1406_entry;
use problem_examples::e14_type_safety::e1407_lossy_float_conversion::e1407_entry;
use problem_examples::e14_type_safety::e1408_unchecked_indexing::e1408_entry;
use problem_examples::e14_type_safety::e1409_partial_initialization::e1409_entry;
use problem_examples::e14_type_safety::e1410_float_equality::e1410_entry;
use problem_examples::e14_type_safety::e1411_type_confusion_transmute::e1411_entry;
use problem_examples::e14_type_safety::e1412_prohibit_unions::e1412_entry;
use problem_examples::e15_concurrency::e1501_non_send_across_threads::e1501_entry;
use problem_examples::e15_concurrency::e1502_lock_across_await::e1502_entry;
use problem_examples::e15_concurrency::e1503_lock_poisoning::e1503_entry;
use problem_examples::e15_concurrency::e1504_interior_mutability_race::e1504_entry;
use problem_examples::e15_concurrency::e1505_non_send_future::e1505_entry;
use problem_examples::e15_concurrency::e1506_deadlock_lock_ordering::e1506_entry;
use problem_examples::e15_concurrency::e1507_unsynchronized_shared_state::e1507_entry;
use problem_examples::e15_concurrency::e1508_sleep_instead_of_sync::e1508_entry;
use problem_examples::e15_concurrency::e1509_channel_lifetime::e1509_entry;
use problem_examples::e15_concurrency::e1510_mutex_instead_of_rwlock::e1510_entry;
use problem_examples::e15_concurrency::e1511_unbounded_spawning::e1511_entry;
use problem_examples::e16_memory_safety::e1601_aliasing_violation::e1601_entry;
use problem_examples::e16_memory_safety::e1602_use_after_free::e1602_entry;
use problem_examples::e16_memory_safety::e1603_dangling_reference::e1603_entry;
use problem_examples::e16_memory_safety::e1604_buffer_overflow::e1604_entry;
use problem_examples::e16_memory_safety::e1605_rc_cycle::e1605_entry;
use problem_examples::e16_memory_safety::e1606_unnecessary_clone::e1606_entry;
use problem_examples::e16_memory_safety::e1607_forget_drop::e1607_entry;
use problem_examples::e16_memory_safety::e1608_double_free::e1608_entry;
use problem_examples::e16_memory_safety::e1609_invalid_slice::e1609_entry;
use problem_examples::e16_memory_safety::e1610_unaligned_deref::e1610_entry;
use problem_examples::e16_memory_safety::e1611_consuming_self::e1611_entry;
use problem_examples::e17_performance::e1701_oversized_struct::e1701_entry;
use problem_examples::e17_performance::e1702_unnecessary_allocations::e1702_entry;
use problem_examples::e17_performance::e1703_string_concat_loop::e1703_entry;
use problem_examples::e17_performance::e1704_unnecessary_collect::e1704_entry;
use problem_examples::e17_performance::e1705_clone_in_hot_path::e1705_entry;
use problem_examples::e17_performance::e1706_non_tail_recursion::e1706_entry;
use problem_examples::e17_performance::e1707_unbounded_recursion::e1707_entry;
use problem_examples::e17_performance::e1708_inefficient_data_structure::e1708_entry;
use problem_examples::e17_performance::e1709_unnecessary_boxing::e1709_entry;
use problem_examples::e17_performance::e1710_large_stack_allocation::e1710_entry;
use problem_examples::e17_performance::e1712_expensive_ops_in_loop::e1712_entry;
use problem_examples::e18_api_design::e1801_glob_imports::e1801_entry;
use problem_examples::e18_api_design::e1802_public_fields::e1802_entry;
use problem_examples::e18_api_design::e1803_bad_naming::e1803_entry;
use problem_examples::e18_api_design::e1804_inconsistent_errors::e1804_entry;
use problem_examples::e18_api_design::e1805_missing_docs::e1805_entry;
use problem_examples::e18_api_design::e1806_internal_details::e1806_entry;
use problem_examples::e18_api_design::e1807_non_idiomatic_builder::e1807_entry;
use problem_examples::e18_api_design::e1808_mutable_getter::e1808_entry;
use problem_examples::e18_api_design::e1809_fallible_new::e1809_entry;
use problem_examples::e18_api_design::e1810_string_instead_of_str::e1810_entry;
use problem_examples::e18_api_design::e1812_non_exhaustive_enum::e1812_entry;

#[derive(Parser)]
#[command(name = "hyp-examples")]
#[command(about = "Explore Rust code problem examples", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all problem categories
    List,

    /// Show details about a specific problem category
    Show {
        /// Category code (e.g., e10, e11, e12)
        category: String,
    },

    /// Run a specific problem example (note: many will panic or have issues!)
    Run {
        /// Problem code (e.g., e101, e102)
        problem: String,
    },

    /// Run ALL problem examples (WARNING: will panic/crash!)
    RunAll,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_categories(),
        Commands::Show { category } => show_category(&category),
        Commands::Run { problem } => run_problem(&problem),
        Commands::RunAll => run_all_problems(),
    }
}

fn list_categories() {
    println!("Available Problem Categories:\n");
    println!("E10* - Unsafe Code Problems");
    println!("  Direct panics, unwrap/expect, unsafe code issues\n");

    println!("E11* - Code Surface Complexity Problems");
    println!("  Cyclomatic complexity, long functions, too many parameters\n");

    println!("E12* - Code Pattern Complexity Problems");
    println!("  Complex generics, lifetimes, GATs, HRTBs, type-level programming\n");

    println!("E13* - Error Handling Problems");
    println!("  Unhandled results, poor error types, panic in Drop\n");

    println!("E14* - Type Safety Problems");
    println!("  Integer overflow, division by zero, type conversions\n");

    println!("E15* - Concurrency Problems");
    println!("  Send/Sync issues, locks, data races, deadlocks\n");

    println!("E16* - Memory Safety Problems");
    println!("  Aliasing, use-after-free, memory leaks, buffer overflow\n");

    println!("E17* - Performance Problems");
    println!("  Large structs, allocations, recursion, inefficient code\n");

    println!("E18* - API Design Problems");
    println!("  Glob imports, public fields, naming, documentation\n");

    println!("Use 'hyp-examples show <category>' for details on a category");
}

fn show_category(category: &str) {
    match category {
        "e10" => {
            println!("E10* - Unsafe Code and Panic Problems\n");
            println!("E1001 - Direct call of panic!() - crashes instead of returning errors");
            println!("E1002 - Direct use of unwrap()/expect() - crashes program on None/Err");
            println!("E1003 - Direct use of unsafe blocks/functions");
            println!("E1004 - Unsafe block without SAFETY comment explaining invariants");
            println!("E1005 - Dereferencing raw pointers without null/alignment checks");
            println!("E1006 - Transmute without verifying size/alignment compatibility");
            println!("E1007 - Potential null pointer dereference");
            println!("E1008 - Unsafe trait implementation (Send/Sync) without justification");
            println!("E1009 - UnsafeCell misuse and interior mutability violations");
            println!("E1010 - Mutable static variable without synchronization");
            println!("E1011 - Uninitialized memory usage (MaybeUninit misuse)");
            println!("E1012 - Unsafe auto trait implementation");
            println!("E1013 - Union type with unsafe field access");
            println!("E1014 - Raw pointer arithmetic (offset/add/sub) without bounds");
            println!("E1015 - Unwrap/expect with poor or missing context message");
            println!("E1016 - Mutex lock().unwrap() - causes panic cascades on poisoning");
            println!("E1017 - todo!/unimplemented! macros in code - panics at runtime");
            println!("E1018 - Prohibit std::mem::transmute unconditionally");
        }
        "e11" => {
            println!("E11* - Code Surface Complexity Problems\n");
            println!("E1101 - High cyclomatic complexity");
            println!("E1102 - Deeply nested logic in loops and conditions");
            println!("E1103 - Too many function parameters");
            println!("E1104 - Overly large struct");
            println!("E1105 - Boolean parameter hell");
            println!("E1106 - Long function (too many lines)");
            println!("E1108 - Deeply nested match expressions");
            println!("E1109 - Excessive method chaining");
            println!("E1110 - Nested callbacks/closures");
            println!("E1111 - Excessive tuple complexity");
            println!("E1112 - Hardcoded magic numbers");
        }
        "e12" => {
            println!("E12* - Code Pattern Complexity Problems\n");
            println!("E1201 - Overly complex generics");
            println!("E1202 - Complex lifetime annotations");
            println!("E1203 - Complicated borrowing patterns");
            println!("E1204 - Trait method ambiguity");
            println!("E1205 - Complex handler with trait bounds");
            println!("E1206 - Deeply nested generics");
            println!("E1207 - Complex user-defined generics");
            println!("E1208 - Phantom types");
            println!("E1209 - Higher-ranked trait bounds (HRTB)");
            println!("E1210 - Recursive types");
            println!("E1211 - Trait object complexity");
            println!("E1212 - Generic Associated Types (GATs)");
            println!("E1213 - Const generic complexity");
            println!("E1214 - Macro-generated code");
            println!("E1215 - Type-level programming");
            println!("E1216 - Chained transformations");
            println!("E1217 - ABBA deadlock pattern");
        }
        "e13" => {
            println!("E13* - Error Handling Problems\n");
            println!("E1301 - Unhandled Result values");
            println!("E1302 - Constructors returning bare values instead of Result");
            println!("E1303 - Ignoring errors with let _ =");
            println!("E1304 - Using unwrap() in error paths");
            println!("E1305 - Non-exhaustive match on Result/Option");
            println!("E1306 - Swallowing errors without logging");
            println!("E1307 - Using String for error types");
            println!("E1308 - Not using ? operator when appropriate");
            println!("E1309 - Panic in Drop implementation");
            println!("E1310 - Error context loss");
        }
        "e14" => {
            println!("E14* - Type Safety Problems\n");
            println!("E1401 - Integer overflow/underflow");
            println!("E1402 - Division by zero");
            println!("E1403 - Modulo by zero");
            println!("E1404 - Narrowing conversions (as)");
            println!("E1405 - Integer division rounding errors");
            println!("E1406 - Signed/unsigned mismatch");
            println!("E1407 - Lossy float to int conversion");
            println!("E1408 - Unchecked array indexing");
            println!("E1409 - Partial initialization");
            println!("E1410 - Float equality comparison with ==");
            println!("E1411 - Type confusion with transmute");
            println!("E1412 - Union types prohibited");
        }
        "e15" => {
            println!("E15* - Concurrency Problems\n");
            println!("E1501 - Non-Send/Sync types crossing thread boundaries");
            println!("E1502 - Holding a lock across .await");
            println!("E1503 - Lock poisoning misuse");
            println!("E1504 - Data race via interior mutability");
            println!("E1505 - Non-Send futures crossing threads");
            println!("E1506 - Deadlock from lock ordering");
            println!("E1507 - Shared mutable state without synchronization");
            println!("E1508 - Using thread::sleep instead of proper synchronization");
            println!("E1509 - Channel sender/receiver lifetime issues");
            println!("E1510 - Arc<Mutex<T>> instead of RwLock");
            println!("E1511 - Unbounded task/thread spawning in loops");
        }
        "e16" => {
            println!("E16* - Memory Safety Problems\n");
            println!("E1601 - Aliasing violations");
            println!("E1602 - Use after free");
            println!("E1603 - Dangling reference");
            println!("E1604 - Buffer overflow");
            println!("E1605 - Memory leak via Rc cycle");
            println!("E1606 - Redundant or dangerous .clone() usage");
            println!("E1607 - Forgetting to drop resources");
            println!("E1608 - Double free");
            println!("E1609 - Slice from raw parts with invalid length");
            println!("E1610 - Unaligned pointer dereference");
            println!("E1611 - Method consumes self unnecessarily");
        }
        "e17" => {
            println!("E17* - Performance Problems\n");
            println!("E1701 - Oversized structs/enums");
            println!("E1702 - Unnecessary allocations in hot path");
            println!("E1703 - String concatenation in loop");
            println!("E1704 - Collecting iterator unnecessarily");
            println!("E1705 - Cloning in hot path");
            println!("E1706 - Recursive function without tail call optimization");
            println!("E1707 - Unbounded recursion");
            println!("E1708 - Inefficient data structure choice");
            println!("E1709 - Unnecessary boxing");
            println!("E1710 - Large stack allocation");
            println!("E1712 - Expensive operations inside loops");
        }
        "e18" => {
            println!("E18* - API Design Problems\n");
            println!("E1801 - Glob imports (use *)");
            println!("E1802 - Public fields without validation");
            println!("E1803 - Naming/API invariant violations");
            println!("E1804 - Inconsistent error types");
            println!("E1805 - Missing documentation on public API");
            println!("E1806 - Exposing internal implementation details");
            println!("E1807 - Non-idiomatic builder pattern");
            println!("E1808 - Mutable getter");
            println!("E1809 - Using new() for fallible construction");
            println!("E1810 - Accepting String instead of &str");
            println!("E1812 - Public enum without #[non_exhaustive]");
        }
        _ => {
            eprintln!("Unknown category: {}", category);
            eprintln!("Available categories: e10, e11, e12, e13, e14, e15, e16, e17, e18");
        }
    }
}

fn run_problem(problem: &str) {
    println!("Running problem: {}", problem);

    let problem_upper = problem.to_uppercase();

    // Build a lookup function using the macro
    let run_fn = || -> Option<Result<(), Box<dyn std::error::Error>>> {
        match problem_upper.as_str() {
            // E10: Unsafe Code
            "E1001" => Some(e1001_entry()),
            "E1002" => Some(e1002_entry()),
            "E1003" => Some(e1003_entry()),
            "E1004" => Some(e1004_entry()),
            "E1005" => Some(e1005_entry()),
            "E1006" => Some(e1006_entry()),
            "E1007" => Some(e1007_entry()),
            "E1008" => Some(e1008_entry()),
            "E1009" => Some(e1009_entry()),
            "E1010" => Some(e1010_entry()),
            "E1011" => Some(e1011_entry()),
            "E1012" => Some(e1012_entry()),
            "E1013" => Some(e1013_entry()),
            "E1014" => Some(e1014_entry()),
            "E1015" => Some(e1015_entry()),
            "E1016" => Some(e1016_entry()),
            "E1017" => Some(e1017_entry()),
            "E1018" => Some(e1018_entry()),

            // E11: Code Surface Complexity
            "E1101" => Some(e1101_entry()),
            "E1102" => Some(e1102_entry()),
            "E1103" => Some(e1103_entry()),
            "E1104" => Some(e1104_entry()),
            "E1105" => Some(e1105_entry()),
            "E1106" => Some(e1106_entry()),
            "E1107" => Some(e1107_entry()),
            "E1108" => Some(e1108_entry()),
            "E1109" => Some(e1109_entry()),
            "E1110" => Some(e1110_entry()),
            "E1111" => Some(e1111_entry()),
            "E1112" => Some(e1112_entry()),

            // E12: Code Pattern Complexity
            "E1201" => Some(e1201_entry()),
            "E1202" => Some(e1202_entry()),
            "E1203" => Some(e1203_entry()),
            "E1204" => Some(e1204_entry()),
            "E1205" => Some(e1205_entry()),
            "E1206" => Some(e1206_entry()),
            "E1207" => Some(e1207_entry()),
            "E1208" => Some(e1208_entry()),
            "E1209" => Some(e1209_entry()),
            "E1210" => Some(e1210_entry()),
            "E1211" => Some(e1211_entry()),
            "E1212" => Some(e1212_entry()),
            "E1213" => Some(e1213_entry()),
            "E1214" => Some(e1214_entry()),
            "E1215" => Some(e1215_entry()),
            "E1216" => Some(e1216_entry()),
            "E1217" => Some(e1217_entry()),

            // E13: Error Handling
            "E1301" => Some(e1301_entry()),
            "E1302" => Some(e1302_entry()),
            "E1303" => Some(e1303_entry()),
            "E1304" => Some(e1304_entry()),
            "E1305" => Some(e1305_entry()),
            "E1306" => Some(e1306_entry()),
            "E1307" => Some(e1307_entry()),
            "E1308" => Some(e1308_entry()),
            "E1309" => Some(e1309_entry()),
            "E1310" => Some(e1310_entry()),

            // E14: Type Safety
            "E1401" => Some(e1401_entry()),
            "E1402" => Some(e1402_entry()),
            "E1403" => Some(e1403_entry()),
            "E1404" => Some(e1404_entry()),
            "E1405" => Some(e1405_entry()),
            "E1406" => Some(e1406_entry()),
            "E1407" => Some(e1407_entry()),
            "E1408" => Some(e1408_entry()),
            "E1409" => Some(e1409_entry()),
            "E1410" => Some(e1410_entry()),
            "E1411" => Some(e1411_entry()),
            "E1412" => Some(e1412_entry()),

            // E15: Concurrency
            "E1501" => Some(e1501_entry()),
            "E1502" => Some(e1502_entry()),
            "E1503" => Some(e1503_entry()),
            "E1504" => Some(e1504_entry()),
            "E1505" => Some(e1505_entry()),
            "E1506" => Some(e1506_entry()),
            "E1507" => Some(e1507_entry()),
            "E1508" => Some(e1508_entry()),
            "E1509" => Some(e1509_entry()),
            "E1510" => Some(e1510_entry()),
            "E1511" => Some(e1511_entry()),

            // E16: Memory Safety
            "E1601" => Some(e1601_entry()),
            "E1602" => Some(e1602_entry()),
            "E1603" => Some(e1603_entry()),
            "E1604" => Some(e1604_entry()),
            "E1605" => Some(e1605_entry()),
            "E1606" => Some(e1606_entry()),
            "E1607" => Some(e1607_entry()),
            "E1608" => Some(e1608_entry()),
            "E1609" => Some(e1609_entry()),
            "E1610" => Some(e1610_entry()),
            "E1611" => Some(e1611_entry()),

            // E17: Performance
            "E1701" => Some(e1701_entry()),
            "E1702" => Some(e1702_entry()),
            "E1703" => Some(e1703_entry()),
            "E1704" => Some(e1704_entry()),
            "E1705" => Some(e1705_entry()),
            "E1706" => Some(e1706_entry()),
            "E1707" => Some(e1707_entry()),
            "E1708" => Some(e1708_entry()),
            "E1709" => Some(e1709_entry()),
            "E1710" => Some(e1710_entry()),
            "E1712" => Some(e1712_entry()),

            // E18: API Design
            "E1801" => Some(e1801_entry()),
            "E1802" => Some(e1802_entry()),
            "E1803" => Some(e1803_entry()),
            "E1804" => Some(e1804_entry()),
            "E1805" => Some(e1805_entry()),
            "E1806" => Some(e1806_entry()),
            "E1807" => Some(e1807_entry()),
            "E1808" => Some(e1808_entry()),
            "E1809" => Some(e1809_entry()),
            "E1810" => Some(e1810_entry()),
            "E1812" => Some(e1812_entry()),

            _ => None,
        }
    };

    match run_fn() {
        Some(Ok(_)) => {
            println!("✓ Completed successfully");
        }
        Some(Err(e)) => {
            println!("✗ Error: {}", e);
            std::process::exit(1);
        }
        None => {
            eprintln!("\n✗ Unknown problem code: {}", problem);
            eprintln!("\nAvailable problems:");
            eprintln!("  E10* - Unsafe Code: e1001-e1018");
            eprintln!("  E11* - Code Complexity: e1101-e1112");
            eprintln!("  E12* - Pattern Complexity: e1201-e1217");
            eprintln!("  E13* - Error Handling: e1301-e1310");
            eprintln!("  E14* - Type Safety: e1401-e1412");
            eprintln!("  E15* - Concurrency: e1501-e1511");
            eprintln!("  E16* - Memory Safety: e1601-e1611");
            eprintln!("  E17* - Performance: e1701-e1712");
            eprintln!("  E18* - API Design: e1801-e1812");
            eprintln!("\nUse 'hyp-examples show <category>' for details");
            std::process::exit(1);
        }
    }
}

// Macro to register all problems and generate both runner and harnesses
macro_rules! define_problems {
    ($macro:ident) => {
        $macro! {
            // E10: Unsafe Code
            ("E1001", "Direct panic", e1001_entry),
            ("E1002", "Direct unwrap/expect crashes", e1002_entry),
            ("E1003", "Unsafe code", e1003_entry),
            ("E1004", "Unsafe without comment", e1004_entry),
            ("E1005", "Raw pointer deref", e1005_entry),
            ("E1006", "Unsafe transmute", e1006_entry),
            ("E1007", "Null pointer deref", e1007_entry),
            ("E1008", "Unsafe trait impl", e1008_entry),
            ("E1009", "Unsafe cell misuse", e1009_entry),
            ("E1010", "Mutable static", e1010_entry),
            ("E1011", "Uninitialized memory", e1011_entry),
            ("E1012", "Unsafe auto trait", e1012_entry),
            ("E1013", "Union field access", e1013_entry),
            ("E1014", "Raw pointer arithmetic", e1014_entry),
            ("E1015", "Unwrap/expect without context", e1015_entry),
            ("E1016", "Mutex unwrap poisoning", e1016_entry),
            ("E1017", "todo!/unimplemented! in code", e1017_entry),
            ("E1018", "Prohibit transmute", e1018_entry),

            // E11: Code Surface Complexity
            ("E1101", "High cyclomatic complexity", e1101_entry),
            ("E1102", "Deeply nested logic in loops and conditions", e1102_entry),
            ("E1103", "Too many params", e1103_entry),
            ("E1104", "Overly large struct", e1104_entry),
            ("E1105", "Boolean parameter hell", e1105_entry),
            ("E1106", "Long function", e1106_entry),
            ("E1107", "Deep nesting", e1107_entry),
            ("E1108", "Nested match", e1108_entry),
            ("E1109", "Excessive chaining", e1109_entry),
            ("E1110", "Nested callbacks", e1110_entry),
            ("E1111", "Excessive tuple complexity", e1111_entry),
            ("E1112", "Hardcoded magic numbers", e1112_entry),

            // E12: Code Pattern Complexity
            ("E1201", "Complex generics", e1201_entry),
            ("E1202", "Complex lifetimes", e1202_entry),
            ("E1203", "Complicated borrowing", e1203_entry),
            ("E1204", "Trait method ambiguity", e1204_entry),
            ("E1205", "Complex handler", e1205_entry),
            ("E1206", "Deeply nested generics", e1206_entry),
            ("E1207", "Complex user generics", e1207_entry),
            ("E1208", "Phantom types", e1208_entry),
            ("E1209", "HRTB", e1209_entry),
            ("E1210", "Recursive types", e1210_entry),
            ("E1211", "Trait object complexity", e1211_entry),
            ("E1212", "GAT complexity", e1212_entry),
            ("E1213", "Const generic complexity", e1213_entry),
            ("E1214", "Macro generated", e1214_entry),
            ("E1215", "Type level programming", e1215_entry),
            ("E1216", "Chained transform", e1216_entry),
            ("E1217", "ABBA deadlock", e1217_entry),

            // E13: Error Handling
            ("E1301", "Unhandled result", e1301_entry),
            ("E1302", "Constructor without result", e1302_entry),
            ("E1303", "Ignored errors", e1303_entry),
            ("E1304", "Unwrap in error path", e1304_entry),
            ("E1305", "Non-exhaustive match", e1305_entry),
            ("E1306", "Swallow errors", e1306_entry),
            ("E1307", "String errors", e1307_entry),
            ("E1308", "Not using question mark", e1308_entry),
            ("E1309", "Panic in drop", e1309_entry),
            ("E1310", "Error context loss", e1310_entry),

            // E14: Type Safety
            ("E1401", "Integer overflow", e1401_entry),
            ("E1402", "Division by zero", e1402_entry),
            ("E1403", "Modulo by zero", e1403_entry),
            ("E1404", "Narrowing conversion", e1404_entry),
            ("E1405", "Integer division rounding", e1405_entry),
            ("E1406", "Signed unsigned mismatch", e1406_entry),
            ("E1407", "Lossy float conversion", e1407_entry),
            ("E1408", "Unchecked indexing", e1408_entry),
            ("E1409", "Partial initialization", e1409_entry),
            ("E1410", "Float equality", e1410_entry),
            ("E1411", "Type confusion transmute", e1411_entry),
            ("E1412", "Union types prohibited", e1412_entry),

            // E15: Concurrency
            ("E1501", "Non-Send across threads", e1501_entry),
            ("E1502", "Lock across await", e1502_entry),
            ("E1503", "Lock poisoning", e1503_entry),
            ("E1504", "Interior mutability race", e1504_entry),
            ("E1505", "Non-Send future", e1505_entry),
            ("E1506", "Deadlock lock ordering", e1506_entry),
            ("E1507", "Unsynchronized shared state", e1507_entry),
            ("E1508", "Sleep instead of sync", e1508_entry),
            ("E1509", "Channel lifetime", e1509_entry),
            ("E1510", "Mutex instead of RwLock", e1510_entry),
            ("E1511", "Unbounded spawning", e1511_entry),

            // E16: Memory Safety
            ("E1601", "Aliasing violations", e1601_entry),
            ("E1602", "Use after free", e1602_entry),
            ("E1603", "Dangling reference", e1603_entry),
            ("E1604", "Buffer overflow", e1604_entry),
            ("E1605", "Rc cycle", e1605_entry),
            ("E1606", "Unnecessary clone", e1606_entry),
            ("E1607", "Forget drop", e1607_entry),
            ("E1608", "Double free", e1608_entry),
            ("E1609", "Invalid slice", e1609_entry),
            ("E1610", "Unaligned deref", e1610_entry),
            ("E1611", "Consuming self unnecessarily", e1611_entry),

            // E17: Performance
            ("E1701", "Oversized struct", e1701_entry),
            ("E1702", "Unnecessary allocations", e1702_entry),
            ("E1703", "String concat loop", e1703_entry),
            ("E1704", "Unnecessary collect", e1704_entry),
            ("E1705", "Clone in hot path", e1705_entry),
            ("E1706", "Non tail recursion", e1706_entry),
            ("E1707", "Unbounded recurions", e1707_entry),
            ("E1708", "Inefficient data structure", e1708_entry),
            ("E1709", "Unnecessary boxing", e1709_entry),
            ("E1710", "Large stack allocation", e1710_entry),
            ("E1712", "Expensive ops in loop", e1712_entry),

            // E18: API Design
            ("E1801", "Glob imports", e1801_entry),
            ("E1802", "Public fields", e1802_entry),
            ("E1803", "Bad naming", e1803_entry),
            ("E1804", "Inconsistent errors", e1804_entry),
            ("E1805", "Missing docs", e1805_entry),
            ("E1806", "Internal details", e1806_entry),
            ("E1807", "Non-idiomatic builder", e1807_entry),
            ("E1808", "Mutable getter", e1808_entry),
            ("E1809", "Fallible new", e1809_entry),
            ("E1810", "String instead of &str", e1810_entry),
            ("E1812", "Non-exhaustive enum", e1812_entry),
        }
    };
}

fn run_all_problems() {
    println!("Running ALL problem examples!\n");

    let mut results = Vec::new();

    // Macro generator for running problems sequentially
    macro_rules! generate_runner {
        ($(($code:expr, $name:expr, $func:ident)),* $(,)?) => {
            $(
                let result = $func();
                let status = match result {
                    Ok(_) => "OK",
                    Err(_) => "ERROR",
                };
                println!("[{}] {} - {}", $code, $name, status);
                results.push(($code.to_string(), $name.to_string(), status.to_string()));
            )*
        };
    }

    define_problems!(generate_runner);

    println!("\n═══════════════════════════════════════");
    println!("Results Summary:");
    println!("═══════════════════════════════════════");

    let ok_count = results.iter().filter(|(_, _, s)| s.contains("OK")).count();
    let panic_count = results
        .iter()
        .filter(|(_, _, s)| s.contains("PANIC") || s.contains("ERROR"))
        .count();

    println!("\nTotal: {} problems executed", results.len());
    println!("  Completed: {}", ok_count);
    println!("  Failed/Panicked:  {}", panic_count);
    println!("═══════════════════════════════════════");
}

#[cfg(kani)] // Only compile this when running Kani
mod verification {
    // Macro generator for creating individual Kani harnesses
    macro_rules! generate_harnesses {
        ($(($code:expr, $name:expr, $func:ident)),* $(,)?) => {
            $(
                #[kani::proof]
                #[kani::unwind(10)] // Increased to 10 to handle loops/recursion
                fn $func() {
                    let _ = super::$func();
                }
            )*
        };
    }

    define_problems!(generate_harnesses);
}

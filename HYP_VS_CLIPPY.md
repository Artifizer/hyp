# Clippy vs hyp-checks-generic: Inverse Comparison

## 1. Clippy Categories Overview

Clippy has **799 lints** organized into these categories:
- **correctness** (deny) - Outright wrong code
- **suspicious** (warn) - Likely bugs
- **style** (warn) - Idiomatic Rust
- **complexity** (warn) - Unnecessarily complex code
- **perf** (warn) - Performance issues
- **pedantic** (allow) - Stricter lints
- **restriction** (allow) - Opinionated lints
- **nursery** (allow) - Experimental lints
- **cargo** (allow) - Cargo.toml issues

---

## 2. Comparison Table: Clippy → hyp-checks-generic

### Correctness Lints (Critical - deny by default)

| Clippy Lint | hyp Equivalent | Status |
|-------------|----------------|--------|
| `approx_constant` | — | **Missing in hyp** |
| `async_yields_async` | — | **Missing in hyp** |
| `bad_bit_mask` | — | **Missing in hyp** |
| `cast_slice_different_sizes` | — | **Missing in hyp** |
| `char_indices_as_byte_indices` | — | **Missing in hyp** |
| `deprecated_semver` | — | **Missing in hyp** |
| `derive_ord_xor_partial_ord` | — | **Missing in hyp** |
| `enum_clike_unportable_variant` | — | **Missing in hyp** |
| `eq_op` | E1905 SuspiciousCode | ✓ Covered |
| `erasing_op` | — | **Missing in hyp** |
| `fn_address_comparisons` | — | **Missing in hyp** |
| `if_let_mutex` | E1502 LockAcrossAwait (partial) | Partial |
| `ifs_same_cond` | E1905 SuspiciousCode | ✓ Covered |
| `ill_formed_doc_comments` | — | **Missing in hyp** |
| `impossible_comparisons` | E1905 SuspiciousCode | ✓ Covered |
| `ineffective_bit_mask` | — | **Missing in hyp** |
| `infinite_iter` | E1707 UnboundedRecursion (partial) | Partial |
| `inherent_to_string_shadow_display` | — | **Missing in hyp** |
| `inline_fn_without_body` | — | **Missing in hyp** |
| `invalid_regex` | — | **Missing in hyp** |
| `invisible_characters` | — | **Missing in hyp** |
| `iter_next_loop` | — | **Missing in hyp** |
| `iter_skip_zero` | — | **Missing in hyp** |
| `let_underscore_lock` | E1502 LockAcrossAwait (partial) | Partial |
| `match_str_case_mismatch` | — | **Missing in hyp** |
| `mem_replace_with_uninit` | E1011 UninitializedMemory | ✓ Covered |
| `min_max` | — | **Missing in hyp** |
| `mistyped_literal_suffixes` | — | **Missing in hyp** |
| `modulo_one` | E1403 ModuloByZero (partial) | Partial |
| `mut_from_ref` | — | **Missing in hyp** |
| `never_loop` | E1905 SuspiciousCode | ✓ Covered |
| `nonsensical_open_options` | — | **Missing in hyp** |
| `not_unsafe_ptr_arg_deref` | E1005 RawPointerDeref | ✓ Covered |
| `option_env_unwrap` | E1002 DirectUnwrapExpect | ✓ Covered |
| `out_of_bounds_indexing` | E1408 UncheckedIndexing | ✓ Covered |
| `overly_complex_bool_expr` | — | **Missing in hyp** |
| `panicking_overflow_checks` | E1401 IntegerOverflow | ✓ Covered |
| `panicking_unwrap` | E1002 DirectUnwrapExpect | ✓ Covered |
| `read_line_without_trim` | — | **Missing in hyp** |
| `recursive_format_impl` | — | **Missing in hyp** |
| `reversed_empty_ranges` | — | **Missing in hyp** |
| `self_assignment` | E1905 SuspiciousCode | ✓ Covered |
| `serde_api_misuse` | — | **Missing in hyp** |
| `size_of_in_element_count` | — | **Missing in hyp** |
| `transmute_null_to_fn` | E1006 UnsafeTransmute | ✓ Covered |
| `transmuting_null` | E1006 UnsafeTransmute | ✓ Covered |
| `uninit_assumed_init` | E1011 UninitializedMemory | ✓ Covered |
| `uninit_vec` | E1011 UninitializedMemory | ✓ Covered |
| `unit_cmp` | — | **Missing in hyp** |
| `unit_hash` | — | **Missing in hyp** |
| `unit_return_expecting_ord` | — | **Missing in hyp** |
| `unsound_collection_transmute` | E1017 ProhibitTransmute | ✓ Covered |
| `unused_io_amount` | E1301 UnhandledResult | ✓ Covered |
| `useless_attribute` | — | **Missing in hyp** |
| `vec_resize_to_zero` | — | **Missing in hyp** |
| `while_immutable_condition` | E1905 SuspiciousCode | ✓ Covered |
| `wrong_transmute` | E1006 UnsafeTransmute | ✓ Covered |
| `zst_offset` | — | **Missing in hyp** |

### Suspicious Lints (warn by default)

| Clippy Lint | hyp Equivalent | Status |
|-------------|----------------|--------|
| `almost_complete_range` | — | **Missing in hyp** |
| `arc_with_non_send_sync` | E1012 UnsafeAutoTrait (partial) | Partial |
| `await_holding_lock` | E1502 LockAcrossAwait | ✓ Covered |
| `await_holding_refcell_ref` | — | **Missing in hyp** |
| `blanket_clippy_restriction_lints` | E1905 InlineDirectives | ✓ Covered |
| `cast_abs_to_unsigned` | E1406 SignedUnsignedMismatch | ✓ Covered |
| `cast_enum_constructor` | — | **Missing in hyp** |
| `cast_enum_truncation` | E1404 NarrowingConversion | ✓ Covered |
| `cast_nan_to_int` | — | **Missing in hyp** |
| `cast_slice_from_raw_parts` | E1609 InvalidSlice | ✓ Covered |
| `cmp_nan` | E1410 FloatEquality (partial) | Partial |
| `collapsible_if` | — | **Missing in hyp** |
| `const_is_empty` | — | **Missing in hyp** |
| `deprecated_clippy_cfg_attr` | — | **Missing in hyp** |
| `doc_link_with_quotes` | — | **Missing in hyp** |
| `drop_non_drop` | E1607 ForgetDrop | ✓ Covered |
| `duplicate_mod` | — | **Missing in hyp** |
| `empty_docs` | E1805 MissingDocumentation | ✓ Covered |
| `empty_loop` | — | **Missing in hyp** |
| `float_cmp` | E1410 FloatEquality | ✓ Covered |
| `float_cmp_const` | E1410 FloatEquality | ✓ Covered |
| `forget_non_drop` | E1607 ForgetDrop | ✓ Covered |
| `four_forward_slashes` | — | **Missing in hyp** |
| `from_raw_with_void_ptr` | E1005 RawPointerDeref | ✓ Covered |
| `iter_out_of_bounds` | E1408 UncheckedIndexing | ✓ Covered |
| `manual_unwrap_or_default` | — | **Missing in hyp** |
| `misrefactored_assign_op` | — | **Missing in hyp** |
| `multi_assignments` | — | **Missing in hyp** |
| `mut_mut` | — | **Missing in hyp** |
| `mutable_key_type` | — | **Missing in hyp** |
| `no_effect` | — | **Missing in hyp** |
| `non_canonical_clone_impl` | — | **Missing in hyp** |
| `non_canonical_partial_ord_impl` | — | **Missing in hyp** |
| `octal_escapes` | — | **Missing in hyp** |
| `path_ends_with_ext` | — | **Missing in hyp** |
| `permissions_set_readonly_false` | — | **Missing in hyp** |
| `suspicious_arithmetic_impl` | — | **Missing in hyp** |
| `suspicious_assignment_formatting` | — | **Missing in hyp** |
| `suspicious_command_arg_space` | — | **Missing in hyp** |
| `suspicious_doc_comments` | — | **Missing in hyp** |
| `suspicious_else_formatting` | — | **Missing in hyp** |
| `suspicious_map` | — | **Missing in hyp** |
| `suspicious_op_assign_impl` | — | **Missing in hyp** |
| `suspicious_to_owned` | — | **Missing in hyp** |
| `suspicious_unary_op_formatting` | — | **Missing in hyp** |
| `zombie_processes` | — | **Missing in hyp** |

### Restriction Lints (Important for strict codebases)

| Clippy Lint | hyp Equivalent | Status |
|-------------|----------------|--------|
| `allow_attributes` | E1905 InlineDirectives | ✓ Covered |
| `allow_attributes_without_reason` | E1905 InlineDirectives | ✓ Covered |
| `arithmetic_side_effects` | E1401 IntegerOverflow | ✓ Covered |
| `as_conversions` | E1404 NarrowingConversion | ✓ Covered |
| `assertions_on_result_states` | — | **Missing in hyp** |
| `clone_on_ref_ptr` | E1606 UnnecessaryClone | ✓ Covered |
| `cognitive_complexity` | E1101 HighCyclomaticComplexity | ✓ Covered |
| `create_dir` | — | **Missing in hyp** |
| `dbg_macro` | — | **Missing in hyp** |
| `decimal_literal_representation` | E1112 MagicNumbers | ✓ Covered |
| `default_numeric_fallback` | — | **Missing in hyp** |
| `deref_by_slicing` | — | **Missing in hyp** |
| `disallowed_script_idents` | — | **Missing in hyp** |
| `else_if_without_else` | — | **Missing in hyp** |
| `empty_drop` | E1607 ForgetDrop | ✓ Covered |
| `empty_enum_variants_with_brackets` | — | **Missing in hyp** |
| `empty_structs_with_brackets` | — | **Missing in hyp** |
| `error_impl_error` | E1307 StringErrorType | ✓ Covered |
| `exhaustive_enums` | E1812 NonExhaustiveEnum | ✓ Covered |
| `exhaustive_structs` | — | **Missing in hyp** |
| `exit` | — | **Missing in hyp** |
| `expect_used` | E1002 DirectUnwrapExpect | ✓ Covered |
| `filetype_is_file` | — | **Missing in hyp** |
| `float_arithmetic` | — | **Missing in hyp** |
| `fn_to_numeric_cast_any` | — | **Missing in hyp** |
| `format_push_string` | E1703 StringConcatInLoop | ✓ Covered |
| `get_unwrap` | E1002 DirectUnwrapExpect | ✓ Covered |
| `if_then_some_else_none` | — | **Missing in hyp** |
| `impl_trait_in_params` | — | **Missing in hyp** |
| `implicit_return` | — | **Missing in hyp** |
| `indexing_slicing` | E1408 UncheckedIndexing | ✓ Covered |
| `infinite_loop` | E1707 UnboundedRecursion | ✓ Covered |
| `inline_asm_x86_att_syntax` | — | **Missing in hyp** |
| `inline_asm_x86_intel_syntax` | — | **Missing in hyp** |
| `integer_division` | E1405 IntegerDivisionRounding | ✓ Covered |
| `integer_division_remainder_used` | — | **Missing in hyp** |
| `iter_over_hash_type` | — | **Missing in hyp** |
| `large_include_file` | — | **Missing in hyp** |
| `let_underscore_drop` | E1303 IgnoredErrors | ✓ Covered |
| `let_underscore_must_use` | E1301 UnhandledResult | ✓ Covered |
| `let_underscore_untyped` | — | **Missing in hyp** |
| `lossy_float_literal` | E1407 LossyFloatConversion | ✓ Covered |
| `map_err_ignore` | E1310 ErrorContextLoss | ✓ Covered |
| `mem_forget` | E1607 ForgetDrop | ✓ Covered |
| `min_ident_chars` | E1803 BadNaming | ✓ Covered |
| `missing_assert_message` | — | **Missing in hyp** |
| `missing_asserts_for_indexing` | E1408 UncheckedIndexing | ✓ Covered |
| `missing_docs_in_private_items` | E1805 MissingDocumentation | ✓ Covered |
| `missing_inline_in_public_items` | — | **Missing in hyp** |
| `missing_trait_methods` | — | **Missing in hyp** |
| `mixed_read_write_in_expression` | — | **Missing in hyp** |
| `modulo_arithmetic` | — | **Missing in hyp** |
| `multiple_inherent_impl` | — | **Missing in hyp** |
| `multiple_unsafe_ops_per_block` | E1908 UnsafeWithoutJustification | ✓ Covered |
| `mutex_atomic` | E1510 MutexInsteadOfRwLock | ✓ Covered |
| `needless_raw_strings` | — | **Missing in hyp** |
| `non_ascii_literal` | — | **Missing in hyp** |
| `panic` | E1001 DirectPanic | ✓ Covered |
| `panic_in_result_fn` | E1309 PanicInDrop | ✓ Covered |
| `partial_pub_fields` | E1802 PublicFields | ✓ Covered |
| `pattern_type_mismatch` | — | **Missing in hyp** |
| `print_stderr` | — | **Missing in hyp** |
| `print_stdout` | — | **Missing in hyp** |
| `pub_use` | E1806 ExposingInternalDetails | ✓ Covered |
| `pub_with_shorthand` | — | **Missing in hyp** |
| `pub_without_shorthand` | — | **Missing in hyp** |
| `question_mark_used` | — | **Missing in hyp** |
| `rc_buffer` | E1605 RcCycle | ✓ Covered |
| `rc_mutex` | — | **Missing in hyp** |
| `redundant_type_annotations` | — | **Missing in hyp** |
| `ref_patterns` | — | **Missing in hyp** |
| `rest_pat_in_fully_bound_structs` | — | **Missing in hyp** |
| `same_name_method` | E1204 TraitMethodAmbiguity | ✓ Covered |
| `self_named_module_files` | E1903 FileLocation | ✓ Covered |
| `semicolon_inside_block` | — | **Missing in hyp** |
| `semicolon_outside_block` | — | **Missing in hyp** |
| `separated_literal_suffix` | — | **Missing in hyp** |
| `shadow_reuse` | — | **Missing in hyp** |
| `shadow_same` | — | **Missing in hyp** |
| `shadow_unrelated` | — | **Missing in hyp** |
| `single_call_fn` | — | **Missing in hyp** |
| `single_char_lifetime_names` | E1202 ComplexLifetimes | ✓ Covered |
| `std_instead_of_alloc` | — | **Missing in hyp** |
| `std_instead_of_core` | — | **Missing in hyp** |
| `str_to_string` | E1810 StringInsteadOfStr | ✓ Covered |
| `string_add` | E1703 StringConcatInLoop | ✓ Covered |
| `string_lit_chars_any` | — | **Missing in hyp** |
| `string_slice` | — | **Missing in hyp** |
| `string_to_string` | — | **Missing in hyp** |
| `suspicious_xor_used_as_pow` | — | **Missing in hyp** |
| `tests_outside_test_module` | — | **Missing in hyp** |
| `todo` | E1004 TodoUnimplemented | ✓ Covered |
| `unsafe_code` | E1003 UnsafeCode | ✓ Covered |
| `try_err` | — | **Missing in hyp** |
| `undocumented_unsafe_blocks` | E1908 UnsafeWithoutJustification | ✓ Covered |
| `unimplemented` | E1004 TodoUnimplemented | ✓ Covered |
| `unnecessary_safety_comment` | — | **Missing in hyp** |
| `unnecessary_safety_doc` | — | **Missing in hyp** |
| `unnecessary_self_imports` | — | **Missing in hyp** |
| `unneeded_field_pattern` | — | **Missing in hyp** |
| `unreachable` | — | **Missing in hyp** |
| `unseparated_literal_suffix` | — | **Missing in hyp** |
| `unwrap_in_result` | E1304 UnwrapInErrorPath | ✓ Covered |
| `unwrap_used` | E1002 DirectUnwrapExpect | ✓ Covered |
| `use_debug` | — | **Missing in hyp** |
| `verbose_file_reads` | — | **Missing in hyp** |
| `wildcard_enum_match_arm` | E1305 NonExhaustiveMatch | ✓ Covered |
| `wildcard_imports` | E1801 GlobImports | ✓ Covered |

### Performance Lints

| Clippy Lint | hyp Equivalent | Status |
|-------------|----------------|--------|
| `box_collection` | E1709 UnnecessaryBoxing | ✓ Covered |
| `boxed_local` | E1709 UnnecessaryBoxing | ✓ Covered |
| `cmp_owned` | — | **Missing in hyp** |
| `expect_fun_call` | — | **Missing in hyp** |
| `extend_with_drain` | — | **Missing in hyp** |
| `format_in_format_args` | — | **Missing in hyp** |
| `iter_nth` | — | **Missing in hyp** |
| `iter_overeager_cloned` | E1606 UnnecessaryClone | ✓ Covered |
| `large_const_arrays` | E1710 LargeStackAllocation | ✓ Covered |
| `large_enum_variant` | E1701 OversizedStructByValue | ✓ Covered |
| `manual_memcpy` | — | **Missing in hyp** |
| `manual_retain` | — | **Missing in hyp** |
| `manual_str_repeat` | — | **Missing in hyp** |
| `map_entry` | — | **Missing in hyp** |
| `needless_collect` | E1704 UnnecessaryCollect | ✓ Covered |
| `or_fun_call` | — | **Missing in hyp** |
| `redundant_allocation` | E1702 UnnecessaryAllocation | ✓ Covered |
| `redundant_clone` | E1606 UnnecessaryClone | ✓ Covered |
| `single_char_pattern` | — | **Missing in hyp** |
| `slow_vector_initialization` | — | **Missing in hyp** |
| `unnecessary_to_owned` | — | **Missing in hyp** |
| `useless_vec` | — | **Missing in hyp** |
| `vec_init_then_push` | — | **Missing in hyp** |

### Complexity Lints

| Clippy Lint | hyp Equivalent | Status |
|-------------|----------------|--------|
| `bind_instead_of_map` | — | **Missing in hyp** |
| `bool_comparison` | — | **Missing in hyp** |
| `borrow_deref_ref` | — | **Missing in hyp** |
| `borrowed_box` | — | **Missing in hyp** |
| `bytes_count_to_len` | — | **Missing in hyp** |
| `char_lit_as_u8` | — | **Missing in hyp** |
| `clone_on_copy` | E1606 UnnecessaryClone | ✓ Covered |
| `crosspointer_transmute` | E1006 UnsafeTransmute | ✓ Covered |
| `double_comparisons` | — | **Missing in hyp** |
| `double_parens` | — | **Missing in hyp** |
| `duration_subsec` | — | **Missing in hyp** |
| `explicit_counter_loop` | — | **Missing in hyp** |
| `explicit_write` | — | **Missing in hyp** |
| `extra_unused_lifetimes` | — | **Missing in hyp** |
| `filter_map_identity` | — | **Missing in hyp** |
| `filter_next` | — | **Missing in hyp** |
| `flat_map_identity` | — | **Missing in hyp** |
| `get_last_with_len` | — | **Missing in hyp** |
| `identity_op` | — | **Missing in hyp** |
| `int_plus_one` | — | **Missing in hyp** |
| `iter_count` | — | **Missing in hyp** |
| `iter_kv_map` | — | **Missing in hyp** |
| `iter_skip_next` | — | **Missing in hyp** |
| `manual_filter` | — | **Missing in hyp** |
| `manual_find` | — | **Missing in hyp** |
| `manual_flatten` | — | **Missing in hyp** |
| `manual_main_separator_str` | — | **Missing in hyp** |
| `manual_map` | — | **Missing in hyp** |
| `manual_range_patterns` | — | **Missing in hyp** |
| `manual_rem_euclid` | — | **Missing in hyp** |
| `manual_slice_size_calculation` | — | **Missing in hyp** |
| `manual_split_once` | — | **Missing in hyp** |
| `manual_strip` | — | **Missing in hyp** |
| `manual_swap` | — | **Missing in hyp** |
| `manual_unwrap_or` | — | **Missing in hyp** |
| `map_flatten` | — | **Missing in hyp** |
| `map_identity` | — | **Missing in hyp** |
| `match_as_ref` | — | **Missing in hyp** |
| `match_single_binding` | — | **Missing in hyp** |
| `needless_arbitrary_self_type` | — | **Missing in hyp** |
| `needless_bool` | — | **Missing in hyp** |
| `needless_bool_assign` | — | **Missing in hyp** |
| `needless_borrowed_reference` | — | **Missing in hyp** |
| `needless_if` | — | **Missing in hyp** |
| `needless_lifetimes` | — | **Missing in hyp** |
| `needless_loop_match` | — | **Missing in hyp** |
| `needless_match` | — | **Missing in hyp** |
| `needless_option_as_deref` | — | **Missing in hyp** |
| `needless_option_take` | — | **Missing in hyp** |
| `needless_question_mark` | — | **Missing in hyp** |
| `needless_splitn` | — | **Missing in hyp** |
| `needless_update` | — | **Missing in hyp** |
| `neg_cmp_op_on_partial_ord` | — | **Missing in hyp** |
| `no_effect` | — | **Missing in hyp** |
| `nonminimal_bool` | — | **Missing in hyp** |
| `only_used_in_recursion` | E1706 NonTailRecursion | ✓ Covered |
| `option_as_ref_deref` | — | **Missing in hyp** |
| `option_filter_map` | — | **Missing in hyp** |
| `option_map_unit_fn` | — | **Missing in hyp** |
| `or_then_unwrap` | — | **Missing in hyp** |
| `overflow_check_conditional` | E1401 IntegerOverflow | ✓ Covered |
| `partialeq_ne_impl` | — | **Missing in hyp** |
| `precedence` | — | **Missing in hyp** |
| `ptr_offset_with_cast` | E1014 RawPointerArithmetic | ✓ Covered |
| `range_minus_one` | — | **Missing in hyp** |
| `range_plus_one` | — | **Missing in hyp** |
| `range_zip_with_len` | — | **Missing in hyp** |
| `redundant_as_str` | — | **Missing in hyp** |
| `redundant_async_block` | — | **Missing in hyp** |
| `redundant_at_rest_pattern` | — | **Missing in hyp** |
| `redundant_closure` | — | **Missing in hyp** |
| `redundant_closure_call` | — | **Missing in hyp** |
| `redundant_guards` | — | **Missing in hyp** |
| `redundant_slicing` | — | **Missing in hyp** |
| `repeat_once` | — | **Missing in hyp** |
| `result_filter_map` | — | **Missing in hyp** |
| `result_map_or_into_option` | — | **Missing in hyp** |
| `result_map_unit_fn` | — | **Missing in hyp** |
| `search_is_some` | — | **Missing in hyp** |
| `seek_from_current` | — | **Missing in hyp** |
| `seek_to_start_instead_of_rewind` | — | **Missing in hyp** |
| `short_circuit_statement` | — | **Missing in hyp** |
| `single_element_loop` | — | **Missing in hyp** |
| `skip_while_next` | — | **Missing in hyp** |
| `string_extend_chars` | — | **Missing in hyp** |
| `strlen_on_c_strings` | — | **Missing in hyp** |
| `temporary_assignment` | — | **Missing in hyp** |
| `too_many_arguments` | E1103 TooManyParameters | ✓ Covered |
| `transmute_bytes_to_str` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_float_to_int` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_int_to_bool` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_int_to_char` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_int_to_float` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_int_to_non_zero` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_num_to_bytes` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_ptr_to_ptr` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_ptr_to_ref` | E1006 UnsafeTransmute | ✓ Covered |
| `transmute_undefined_repr` | E1006 UnsafeTransmute | ✓ Covered |
| `transmutes_expressible_as_ptr_casts` | E1006 UnsafeTransmute | ✓ Covered |
| `type_complexity` | E1201 ComplexGenerics | ✓ Covered |
| `unit_arg` | — | **Missing in hyp** |
| `unnecessary_cast` | — | **Missing in hyp** |
| `unnecessary_filter_map` | — | **Missing in hyp** |
| `unnecessary_find_map` | — | **Missing in hyp** |
| `unnecessary_literal_unwrap` | — | **Missing in hyp** |
| `unnecessary_map_on_constructor` | — | **Missing in hyp** |
| `unnecessary_operation` | — | **Missing in hyp** |
| `unnecessary_sort_by` | — | **Missing in hyp** |
| `unnecessary_unwrap` | — | **Missing in hyp** |
| `unneeded_wildcard_pattern` | — | **Missing in hyp** |
| `useless_asref` | — | **Missing in hyp** |
| `useless_conversion` | — | **Missing in hyp** |
| `useless_format` | — | **Missing in hyp** |
| `useless_transmute` | E1006 UnsafeTransmute | ✓ Covered |
| `vec_box` | E1709 UnnecessaryBoxing | ✓ Covered |
| `while_let_loop` | — | **Missing in hyp** |
| `wildcard_in_or_patterns` | — | **Missing in hyp** |
| `zero_divided_by_zero` | E1402 DivisionByZero | ✓ Covered |
| `zero_prefixed_literal` | — | **Missing in hyp** |

---

## 3. What is Missing in hyp-checks-generic

### High Priority Gaps (Correctness/Suspicious)

| Category | Missing Clippy Lints | Count |
|----------|---------------------|-------|
| **Correctness** | `approx_constant`, `async_yields_async`, `bad_bit_mask`, `eq_op`, `erasing_op`, `ifs_same_cond`, `impossible_comparisons`, `infinite_iter`, `invalid_regex`, `never_loop`, `self_assignment`, `unit_cmp`, `while_immutable_condition` | ~40 |
| **Suspicious** | `almost_complete_range`, `await_holding_refcell_ref`, `cast_nan_to_int`, `empty_loop`, `misrefactored_assign_op`, `mut_mut`, `mutable_key_type`, `no_effect`, `octal_escapes`, `suspicious_arithmetic_impl`, `zombie_processes` | ~50 |

### Medium Priority Gaps (Performance/Complexity)

| Category | Missing Clippy Lints | Count |
|----------|---------------------|-------|
| **Performance** | `cmp_owned`, `expect_fun_call`, `iter_nth`, `manual_memcpy`, `or_fun_call`, `single_char_pattern`, `slow_vector_initialization`, `useless_vec`, `vec_init_then_push` | ~25 |
| **Complexity** | `bind_instead_of_map`, `bool_comparison`, `manual_*` family (~20 lints), `needless_*` family (~15 lints), `redundant_*` family (~10 lints) | ~100 |

### Low Priority Gaps (Style/Pedantic)

| Category | Missing Clippy Lints | Count |
|----------|---------------------|-------|
| **Style** | Most style lints (naming, formatting, idioms) | ~150 |
| **Pedantic** | Most pedantic lints | ~100 |

---

## 4. What is Missing in Clippy (hyp-only checkers)

| hyp Code | Checker Name | Why Clippy Doesn't Have It |
|----------|--------------|---------------------------|
| **E1005** | RawPointerDeref | Clippy has partial coverage via `not_unsafe_ptr_arg_deref` |
| **E1007** | NullPointerDeref | No direct equivalent - hyp detects null pointer patterns |
| **E1008** | UnsafeTraitImpl | No direct equivalent - flags unsafe trait implementations |
| **E1009** | UnsafeCellMisuse | No direct equivalent - detects UnsafeCell anti-patterns |
| **E1010** | MutableStatic | Partial via compiler warnings, hyp is stricter |
| **E1012** | UnsafeAutoTrait | No direct equivalent - flags unsafe Send/Sync impls |
| **E1013** | UnionFieldAccess | No direct equivalent - detects union field reads |
| **E1014** | RawPointerArithmetic | Partial via `ptr_offset_with_cast` |
| **E1015** | UnwrapExpect w/o context | Stricter than `expect_used` |
| **E1016** | MutexUnwrap | Specific `mutex.lock().unwrap()` detection |
| **E1102** | DeeplyNestedLogic | Specific nesting depth (not just cognitive complexity) |
| **E1103** | TooManyParameters | Stricter than clippy's `too_many_arguments` |
| **E1104** | LargeStruct | Struct field count check |
| **E1105** | BooleanParameterHell | Multiple boolean params anti-pattern |
| **E1106** | LongFunction | Line count based |
| **E1107** | DeeplyNestedConditionals | Specific conditional nesting |
| **E1108** | DeeplyNestedMatch | Match nesting depth |
| **E1109** | ExcessiveChaining | Method chain length |
| **E1110** | DeeplyNestedClosures | Closure nesting depth |
| **E1111** | ExcessiveTupleComplexity | Tuple element count |
| **E1203** | ComplicatedBorrowing | Borrow pattern analysis |
| **E1204** | TraitMethodAmbiguity | Detects ambiguous trait methods |
| **E1205** | NestedTraitBounds | Trait bound nesting |
| **E1206** | NestedGenerics | Generic nesting depth |
| **E1207** | ComplexConstraints | Where clause complexity |
| **E1208** | PhantomTypes | PhantomData usage detection |
| **E1209** | HRTB | Higher-ranked trait bounds |
| **E1210** | RecursiveTypes | Recursive type definitions |
| **E1211** | TraitObjectComplexity | dyn trait complexity |
| **E1212** | GATComplexity | Generic associated types |
| **E1213** | ConstGenericComplexity | Const generics |
| **E1214** | MacroImpl | Macro-generated impls |
| **E1215** | TypeLevelProgramming | Type-level computation |
| **E1216** | AssociatedTypeChains | Associated type depth |
| **E1217** | ABBADeadlock | Lock ordering analysis |
| **E1302** | ConstructorWithoutResult | Fallible constructor detection |
| **E1303** | IgnoredErrors | Detects `let _ = result` patterns |
| **E1304** | UnwrapInErrorPath | Unwrap in error handling code |
| **E1306** | SwallowedErrors | Error suppression detection |
| **E1307** | StringErrorType | Discourages `Result<T, String>` |
| **E1308** | NotUsingQuestionMark | Enforces ? operator usage |
| **E1309** | PanicInDrop | Panic in Drop implementation |
| **E1310** | ErrorContextLoss | Error chain preservation |
| **E1402** | DivisionByZero | Division safety |
| **E1403** | ModuloByZero | Modulo safety |
| **E1405** | IntegerDivisionRounding | Integer division awareness |
| **E1409** | PartialInitialization | Struct initialization |
| **E1412** | ProhibitUnions | Can ban union types |
| **E1413** | 64BitIntegerOverflow | Specific to 64-bit |
| **E1503** | LockPoisoning | Poison handling |
| **E1506** | DeadlockLockOrdering | Lock order analysis |
| **E1508** | SleepInsteadOfSync | Sleep-based synchronization |
| **E1509** | ChannelLifetime | Channel usage patterns |
| **E1510** | MutexInsteadOfRwLock | Lock type selection |
| **E1511** | UnboundedSpawning | Task spawning limits |
| **E1512** | ProhibitStdThreadSpawn | Prohibits std::thread::spawn, suggests tokio::task::spawn_blocking |
| **E1513** | BlockingSyscallsAsync | Detects blocking syscalls (std::fs, std::net, std::thread::sleep) in async code |
| **E1603** | DanglingReference | Additional patterns beyond compiler |
| **E1604** | BufferOverflow | Buffer access patterns |
| **E1609** | InvalidSlice | Slice construction |
| **E1610** | UnalignedDeref | Alignment issues |
| **E1611** | ConsumingSelf | Self-consumption patterns |
| **E1612** | ProhibitCustomAllocators | Detects #[global_allocator] usage, microservices should use default allocator |
| **E1705** | CloneInHotPath | Hot path analysis |
| **E1706** | NonTailRecursion | Recursion optimization |
| **E1707** | UnboundedRecursion | Recursion depth |
| **E1708** | InefficientDataStructure | Data structure selection |
| **E1712** | ExpensiveOpsInLoop | Loop optimization |
| **E1801** | GlobImports | Glob/wildcard import restrictions |
| **E1802** | PublicFields | Encapsulation enforcement |
| **E1804** | InconsistentErrorTypes | Error type consistency |
| **E1806** | ExposingInternalDetails | API boundary enforcement |
| **E1807** | NonIdiomaticBuilder | Builder pattern validation |
| **E1808** | MutableGetter | Getter pattern |
| **E1809** | FallibleNew | Constructor patterns |
| **E1901** | CriticalLintOverride | Detects overrides of critical safety lints (unsafe_code, panic, unwrap_used) |
| **E1902** | MediumLintOverride | Detects overrides of medium lints (performance, complexity) |
| **E1903** | MinorLintOverride | Detects overrides of minor/style lints |
| **E1904** | AllowedNames | Naming whitelist (compliance category) |
| **E1905** | SuspiciousCode | Detects suspicious patterns: `eq_op`, `ifs_same_cond`, `self_assignment`, `never_loop`, `while_immutable_condition`, `impossible_comparisons` |
| **E1906** | FileLocation | File organization rules (compliance category) |
| **E1908** | UnsafeJustification | Stricter justification format (compliance category) |

---

## 5. TODO: Priority Gap List for hyp to Overcome Clippy

### Tier 1: Critical (Correctness bugs clippy catches that hyp doesn't)

1. **`invalid_regex`** - Invalid regex patterns
2. **`approx_constant`** - Using approximate values instead of constants (e.g., 3.14 vs PI)
3. **`bad_bit_mask`** - Incorrect bit masking operations
4. **`async_yields_async`** - Async block yielding another async

### Tier 2: High Priority (Suspicious patterns)

1. **`await_holding_refcell_ref`** - Holding RefCell guard across await
2. **`mutable_key_type`** - Using mutable types as hash keys
3. **`suspicious_arithmetic_impl`** - Suspicious Add/Sub/Mul/Div implementations
4. **`zombie_processes`** - Not waiting on child processes
5. **`empty_loop`** - Empty loop bodies
6. **`no_effect`** - Statements with no effect
7. **`mut_mut`** - `&mut &mut` references
8. **`cast_nan_to_int`** - Casting NaN to integer

### Tier 3: Medium Priority (Performance/Quality)

1. **`or_fun_call`** - Using `unwrap_or(expensive())` instead of `unwrap_or_else`
2. **`expect_fun_call`** - Using `expect(format!())` instead of `expect_with`
3. **`slow_vector_initialization`** - Inefficient vector initialization
4. **`manual_memcpy`** - Manual byte copying instead of memcpy
5. **`single_char_pattern`** - Using string for single char search
6. **`useless_vec`** - Creating vec when slice would suffice
7. **`cmp_owned`** - Unnecessary owned comparison


---

## Summary

| Metric | Count |
|--------|-------|
| **Clippy total lints** | 799 |
| **hyp-checks-generic checkers** | 117 |
| **Clippy lints covered by hyp** | ~40 |
| **Clippy lints missing in hyp** | ~760 |
| **hyp-only checkers (not in clippy)** | 82 |
| **Critical gaps to close** | ~12 (Tier 1 + Tier 2) |

TODO: **Key insight**: hyp-checks-generic is not trying to replace clippy—it's designed to be **complementary** with:
1. **Un-overridable enforcement** (no `#[allow]`)
2. **Stricter variants** of existing checks
3. **Unique checkers** for patterns clippy doesn't cover (concurrency, complexity metrics, API design)

To "overcome" clippy, hyp should focus on the **Tier 1 correctness bugs** first, as these are the most impactful and hardest to justify suppressing anyway.

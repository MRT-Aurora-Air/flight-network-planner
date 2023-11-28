#![warn(
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::checked_conversions,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::cloned_instead_of_copied,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::copy_iterator,
    clippy::create_dir,
    clippy::default_trait_access,
    clippy::deref_by_slicing,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::from_iter_instead_of_collect,
    clippy::future_not_send,
    clippy::get_unwrap,
    clippy::if_not_else,
    clippy::if_then_some_else_none,
    clippy::implicit_hasher,
    clippy::impl_trait_in_params,
    clippy::imprecise_flops,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::many_single_char_names,
    clippy::map_err_ignore,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::mismatching_type_param_order,
    clippy::missing_assert_message,
    clippy::missing_const_for_fn,
    clippy::missing_enforced_import_renames,
    clippy::multiple_unsafe_ops_per_block,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::non_ascii_literal,
    clippy::non_send_fields_in_send_ty,
    clippy::or_fun_call,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::rc_buffer,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::redundant_feature_names,
    clippy::redundant_pub_crate,
    clippy::ref_option_ref,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_self_not_must_use,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::semicolon_inside_block,
    clippy::separated_literal_suffix,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::single_match_else,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_slice,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_ptr_to_ptr,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::trivially_copy_pass_by_ref,
    clippy::try_err,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::unused_self,
    clippy::unwrap_in_result,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_bit_mask,
    clippy::verbose_file_reads
)]
#![deny(
    clippy::derive_partial_eq_without_eq,
    clippy::match_bool,
    clippy::mem_forget,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::path_buf_push_overwrite,
    clippy::rc_mutex,
    clippy::wildcard_dependencies
)]

mod cmd;
mod types;
mod utils;

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use clap_complete_fig::Fig;
use itertools::Itertools;
use log::warn;
use types::config::Config;

use crate::{
    cmd::{run, stats, update},
    types::flight_data::FlightData,
};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Parser)]
enum Command {
    /// Run the planner
    Run(Run),
    /// Gets the configuration for the planner
    GetConfig,
    /// Tool to format the output of `run` as a mapping of gates to destinations
    GateKeys(GateKeys),
    /// Generate a completion file for your shell
    Completion(Completion),
}

#[derive(Parser)]
struct Run {
    /// The configuration YML file to read from
    file: PathBuf,
    /// Whether to print statistics
    #[clap(short, long, action)]
    stats: bool,
    /// The old output file
    /// (will be used to preserve original flight routes so it won't duplicate so much)
    #[clap(long, value_parser)]
    old: Option<PathBuf>,
}

#[derive(Parser)]
struct GateKeys {
    /// The output file from `run`
    #[clap(default_value = "out.txt")]
    out_file: PathBuf,
}

#[derive(Parser)]
struct Completion {
    /// The shell to generate for
    #[arg(value_enum)]
    shell: Shell,
    /// Whether to generate for Fig instead
    #[clap(short, long, action)]
    fig: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let args = Args::parse();
    match args.command {
        Command::Run(run) => {
            let mut config: Config = serde_yaml::from_reader(std::fs::File::open(&run.file)?)?;
            let mut fd = FlightData::from_sheets()?;
            fd.preprocess(&mut config)?;
            let old_plan = if let Some(old) = &run.old {
                Some(update::load_from_out(old.to_owned())?)
            } else {
                None
            };
            let mut result = run::run(&mut config, &fd, &old_plan)?;
            if run.stats {
                eprintln!("\n{}", stats::get_stats(&result, &mut config)?);
            }
            if let Some(old) = &run.old {
                result = update::update(old.to_owned(), result, &mut config)?;
            }
            let res = result
                .into_iter()
                .sorted_by_key(|f| f.flight_number)
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            println!("{res}");
        }
        Command::GetConfig => {
            println!("{}", include_str!("../data/default_config.yml"));
        }
        Command::GateKeys(gate_keys) => {
            let flights = update::load_from_out(gate_keys.out_file)?;
            let mut map: HashMap<_, Vec<_>> = HashMap::new();
            for flight in flights {
                map.entry(flight.airport1)
                    .or_default()
                    .push((flight.airport2, flight.flight_number));
            }
            let res = map
                .iter()
                .map(|((ka, kg), vs)| {
                    format!(
                        "{} {}: {}",
                        ka,
                        kg,
                        vs.iter()
                            .map(|((va, vg), num)| format!("{num} {va} {vg}"))
                            .join(", ")
                    )
                })
                .sorted()
                .join("\n");
            println!("{res}");
        }
        Command::Completion(completion) => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_owned();
            if completion.fig {
                clap_complete::generate(Fig, &mut cmd, name, &mut std::io::stdout());
            } else {
                clap_complete::generate(completion.shell, &mut cmd, name, &mut std::io::stdout());
            }
        }
    }
    Ok(())
}

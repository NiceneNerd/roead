var srcIndex = new Map(JSON.parse('[\
["acid_io",["",[],["lib.rs"]]],\
["almost",["",[],["imp.rs","lib.rs"]]],\
["array_init",["",[],["lib.rs"]]],\
["auto_enum",["",[],["common.rs","enum_flags.rs","int_enums.rs","int_eval.rs","lib.rs"]]],\
["base64",["",[["engine",[["general_purpose",[],["decode.rs","decode_suffix.rs","mod.rs"]]],["mod.rs"]],["read",[],["decoder.rs","mod.rs"]],["write",[],["encoder.rs","encoder_string_writer.rs","mod.rs"]]],["alphabet.rs","chunked_encoder.rs","decode.rs","display.rs","encode.rs","lib.rs","prelude.rs"]]],\
["binrw",["",[["binread",[],["impls.rs","mod.rs"]],["binwrite",[],["impls.rs","mod.rs"]],["error",[],["backtrace.rs","mod.rs"]],["io",[],["bufreader.rs","mod.rs","prelude.rs","seek.rs","take_seek.rs"]]],["docs.rs","endian.rs","file_ptr.rs","helpers.rs","lib.rs","meta.rs","named_args.rs","pos_value.rs","private.rs","punctuated.rs","strings.rs"]]],\
["binrw_derive",["",[["binrw",[["backtrace",[],["mod.rs","syntax_highlighting.rs"]],["codegen",[["read_options",[],["enum.rs","map.rs","struct.rs"]],["write_options",[],["enum.rs","prelude.rs","struct.rs","struct_field.rs"]]],["meta.rs","mod.rs","read_options.rs","sanitization.rs","write_options.rs"]],["parser",[["types",[],["assert.rs","cond_endian.rs","condition.rs","enum_error_mode.rs","err_context.rs","field_mode.rs","imports.rs","magic.rs","map.rs","mod.rs","passed_args.rs","spanned_value.rs"]]],["attrs.rs","field_level_attrs.rs","keywords.rs","macros.rs","mod.rs","top_level_attrs.rs","try_set.rs"]]],["combiner.rs","mod.rs"]],["fn_helper",[],["mod.rs"]],["named_args",[],["codegen.rs","mod.rs"]]],["lib.rs","meta_types.rs","result.rs","util.rs"]]],\
["bytemuck",["",[],["anybitpattern.rs","checked.rs","contiguous.rs","internal.rs","lib.rs","no_uninit.rs","offset_of.rs","pod.rs","pod_in_option.rs","transparent.rs","zeroable.rs","zeroable_in_option.rs"]]],\
["byteorder",["",[],["io.rs","lib.rs"]]],\
["cfg_if",["",[],["lib.rs"]]],\
["cxx",["",[["macros",[],["assert.rs","mod.rs"]],["symbols",[],["exception.rs","mod.rs","rust_slice.rs","rust_str.rs","rust_string.rs","rust_vec.rs"]]],["cxx_string.rs","cxx_vector.rs","exception.rs","extern_type.rs","fmt.rs","function.rs","hash.rs","lib.rs","lossy.rs","memory.rs","opaque.rs","result.rs","rust_slice.rs","rust_str.rs","rust_string.rs","rust_type.rs","rust_vec.rs","shared_ptr.rs","type_id.rs","unique_ptr.rs","unwind.rs","vector.rs","weak_ptr.rs"]]],\
["cxxbridge_macro",["",[["syntax",[],["atom.rs","attrs.rs","cfg.rs","check.rs","derive.rs","discriminant.rs","doc.rs","error.rs","file.rs","ident.rs","impls.rs","improper.rs","instantiate.rs","mangle.rs","map.rs","mod.rs","names.rs","namespace.rs","parse.rs","pod.rs","qualified.rs","report.rs","resolve.rs","set.rs","symbol.rs","tokens.rs","toposort.rs","trivial.rs","types.rs","visit.rs"]]],["derive.rs","expand.rs","generics.rs","lib.rs","tokens.rs","type_id.rs"]]],\
["either",["",[],["into_either.rs","iterator.rs","lib.rs"]]],\
["equivalent",["",[],["lib.rs"]]],\
["hashbrown",["",[["external_trait_impls",[],["mod.rs"]],["raw",[],["alloc.rs","bitmask.rs","mod.rs","sse2.rs"]]],["lib.rs","macros.rs","map.rs","scopeguard.rs","set.rs","table.rs"]]],\
["indexmap",["",[["map",[["core",[],["entry.rs","raw.rs","raw_entry_v1.rs"]]],["core.rs","iter.rs","mutable.rs","serde_seq.rs","slice.rs"]],["set",[],["iter.rs","mutable.rs","slice.rs"]]],["arbitrary.rs","lib.rs","macros.rs","map.rs","serde.rs","set.rs","util.rs"]]],\
["itoa",["",[],["lib.rs","udiv128.rs"]]],\
["join_str",["",[],["lib.rs"]]],\
["lexical",["",[],["lib.rs"]]],\
["lexical_core",["",[],["lib.rs"]]],\
["lexical_parse_float",["",[],["api.rs","bigint.rs","binary.rs","float.rs","index.rs","lemire.rs","lib.rs","limits.rs","mask.rs","number.rs","options.rs","parse.rs","shared.rs","slow.rs","table.rs","table_binary.rs","table_decimal.rs","table_large.rs","table_lemire.rs","table_small.rs"]]],\
["lexical_parse_integer",["",[],["algorithm.rs","api.rs","lib.rs","options.rs","parse.rs","shared.rs"]]],\
["lexical_util",["",[],["algorithm.rs","api.rs","ascii.rs","assert.rs","constants.rs","digit.rs","div128.rs","error.rs","extended_float.rs","format.rs","format_builder.rs","format_flags.rs","iterator.rs","lib.rs","mul.rs","noskip.rs","not_feature_format.rs","num.rs","options.rs","result.rs","step.rs"]]],\
["lexical_write_float",["",[],["algorithm.rs","api.rs","binary.rs","float.rs","hex.rs","index.rs","lib.rs","options.rs","shared.rs","table.rs","table_dragonbox.rs","write.rs"]]],\
["lexical_write_integer",["",[],["algorithm.rs","api.rs","decimal.rs","index.rs","lib.rs","options.rs","radix.rs","table.rs","table_binary.rs","table_decimal.rs","write.rs"]]],\
["libc",["",[["unix",[["linux_like",[["linux",[["arch",[["generic",[],["mod.rs"]]],["mod.rs"]],["gnu",[["b64",[["x86_64",[],["align.rs","mod.rs","not_x32.rs"]]],["mod.rs"]]],["align.rs","mod.rs"]]],["align.rs","mod.rs","non_exhaustive.rs"]]],["mod.rs"]]],["align.rs","mod.rs"]]],["fixed_width_ints.rs","lib.rs","macros.rs"]]],\
["link_cplusplus",["",[],["lib.rs"]]],\
["lock_api",["",[],["lib.rs","mutex.rs","remutex.rs","rwlock.rs"]]],\
["memchr",["",[["arch",[["all",[["packedpair",[],["default_rank.rs","mod.rs"]]],["memchr.rs","mod.rs","rabinkarp.rs","shiftor.rs","twoway.rs"]],["generic",[],["memchr.rs","mod.rs","packedpair.rs"]],["x86_64",[["avx2",[],["memchr.rs","mod.rs","packedpair.rs"]],["sse2",[],["memchr.rs","mod.rs","packedpair.rs"]]],["memchr.rs","mod.rs"]]],["mod.rs"]],["memmem",[],["mod.rs","searcher.rs"]]],["cow.rs","ext.rs","lib.rs","macros.rs","memchr.rs","vector.rs"]]],\
["num_integer",["",[],["average.rs","lib.rs","roots.rs"]]],\
["num_traits",["",[["ops",[],["bytes.rs","checked.rs","euclid.rs","inv.rs","mod.rs","mul_add.rs","overflowing.rs","saturating.rs","wrapping.rs"]]],["bounds.rs","cast.rs","float.rs","identities.rs","int.rs","lib.rs","macros.rs","pow.rs","real.rs","sign.rs"]]],\
["owo_colors",["",[["colors",[],["css.rs","custom.rs","dynamic.rs","xterm.rs"]]],["colors.rs","combo.rs","dyn_colors.rs","dyn_styles.rs","lib.rs","styled_list.rs","styles.rs"]]],\
["parking_lot",["",[],["condvar.rs","deadlock.rs","elision.rs","fair_mutex.rs","lib.rs","mutex.rs","once.rs","raw_fair_mutex.rs","raw_mutex.rs","raw_rwlock.rs","remutex.rs","rwlock.rs","util.rs"]]],\
["parking_lot_core",["",[["thread_parker",[],["linux.rs","mod.rs"]]],["lib.rs","parking_lot.rs","spinwait.rs","util.rs","word_lock.rs"]]],\
["roead",["",[["aamp",[],["mod.rs","names.rs","parser.rs","text.rs","writer.rs"]],["byml",[],["mod.rs","parser.rs","text.rs","writer.rs"]],["sarc",[],["mod.rs","parse.rs","write.rs"]]],["lib.rs","types.rs","util.rs","yaml.rs","yaz0.rs"]]],\
["rustc_hash",["",[],["lib.rs","seeded_state.rs"]]],\
["ryml",["",[],["inner.rs","lib.rs","node.rs"]]],\
["ryu",["",[["buffer",[],["mod.rs"]],["pretty",[],["exponent.rs","mantissa.rs","mod.rs"]]],["common.rs","d2s.rs","d2s_full_table.rs","d2s_intrinsics.rs","digit_table.rs","f2s.rs","f2s_intrinsics.rs","lib.rs"]]],\
["scc",["",[["hash_table",[],["bucket.rs","bucket_array.rs"]],["tree_index",[],["internal_node.rs","leaf.rs","leaf_node.rs","node.rs"]]],["bag.rs","exit_guard.rs","hash_cache.rs","hash_index.rs","hash_map.rs","hash_set.rs","hash_table.rs","lib.rs","linked_list.rs","queue.rs","stack.rs","tree_index.rs","wait_queue.rs"]]],\
["scopeguard",["",[],["lib.rs"]]],\
["sdd",["",[],["atomic_owned.rs","atomic_shared.rs","collectible.rs","collector.rs","epoch.rs","exit_guard.rs","guard.rs","lib.rs","owned.rs","ptr.rs","ref_counted.rs","shared.rs","tag.rs"]]],\
["serde",["",[["de",[],["format.rs","ignored_any.rs","impls.rs","mod.rs","seed.rs","size_hint.rs","value.rs"]],["private",[],["de.rs","doc.rs","mod.rs","ser.rs"]],["ser",[],["fmt.rs","impls.rs","impossible.rs","mod.rs"]]],["integer128.rs","lib.rs","macros.rs"]]],\
["serde_derive",["",[["internals",[],["ast.rs","attr.rs","case.rs","check.rs","ctxt.rs","mod.rs","receiver.rs","respan.rs","symbol.rs"]]],["bound.rs","de.rs","dummy.rs","fragment.rs","lib.rs","pretend.rs","ser.rs","this.rs"]]],\
["serde_json",["",[["io",[],["mod.rs"]],["value",[],["de.rs","from.rs","index.rs","mod.rs","partial_eq.rs","ser.rs"]]],["de.rs","error.rs","iter.rs","lib.rs","macros.rs","map.rs","number.rs","read.rs","ser.rs"]]],\
["smallvec",["",[],["lib.rs"]]],\
["smartstring",["",[],["boxed.rs","casts.rs","config.rs","inline.rs","iter.rs","lib.rs","marker_byte.rs","ops.rs","serde.rs"]]],\
["static_assertions",["",[],["assert_cfg.rs","assert_eq_align.rs","assert_eq_size.rs","assert_fields.rs","assert_impl.rs","assert_obj_safe.rs","assert_trait.rs","assert_type.rs","const_assert.rs","lib.rs"]]],\
["thiserror",["",[],["aserror.rs","display.rs","lib.rs","provide.rs"]]],\
["thiserror_impl",["",[],["ast.rs","attr.rs","expand.rs","fmt.rs","generics.rs","lib.rs","prop.rs","span.rs","valid.rs"]]],\
["thiserror_impl_no_std",["",[],["ast.rs","attr.rs","expand.rs","fmt.rs","generics.rs","lib.rs","prop.rs","valid.rs"]]],\
["thiserror_no_std",["",[],["aserror.rs","display.rs","lib.rs"]]]\
]'));
createSrcSidebar();
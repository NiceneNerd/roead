(function() {var type_impls = {
"scc":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AsRef%3CHashMap%3CK,+V,+H%3E%3E-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2185-2194\">source</a><a href=\"#impl-AsRef%3CHashMap%3CK,+V,+H%3E%3E-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'h, K, V, H&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"struct\" href=\"scc/hash_map/struct.HashMap.html\" title=\"struct scc::hash_map::HashMap\">HashMap</a>&lt;K, V, H&gt;&gt; for <a class=\"struct\" href=\"scc/hash_map/struct.Reserve.html\" title=\"struct scc::hash_map::Reserve\">Reserve</a>&lt;'h, K, V, H&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_ref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2191-2193\">source</a><a href=\"#method.as_ref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html#tymethod.as_ref\" class=\"fn\">as_ref</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"scc/hash_map/struct.HashMap.html\" title=\"struct scc::hash_map::HashMap\">HashMap</a>&lt;K, V, H&gt;</h4></section></summary><div class='docblock'>Converts this type into a shared reference of the (usually inferred) input type.</div></details></div></details>","AsRef<HashMap<K, V, H>>","scc::hash_set::Reserve"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2196-2205\">source</a><a href=\"#impl-Debug-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'h, K, V, H&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"scc/hash_map/struct.Reserve.html\" title=\"struct scc::hash_map::Reserve\">Reserve</a>&lt;'h, K, V, H&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2202-2204\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","scc::hash_set::Reserve"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deref-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2207-2218\">source</a><a href=\"#impl-Deref-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'h, K, V, H&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"scc/hash_map/struct.Reserve.html\" title=\"struct scc::hash_map::Reserve\">Reserve</a>&lt;'h, K, V, H&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Target\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Target\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" class=\"associatedtype\">Target</a> = <a class=\"struct\" href=\"scc/hash_map/struct.HashMap.html\" title=\"struct scc::hash_map::HashMap\">HashMap</a>&lt;K, V, H&gt;</h4></section></summary><div class='docblock'>The resulting type after dereferencing.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2215-2217\">source</a><a href=\"#method.deref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#tymethod.deref\" class=\"fn\">deref</a>(&amp;self) -&gt; &amp;Self::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html#associatedtype.Target\" title=\"type core::ops::deref::Deref::Target\">Target</a></h4></section></summary><div class='docblock'>Dereferences the value.</div></details></div></details>","Deref","scc::hash_set::Reserve"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Drop-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2220-2234\">source</a><a href=\"#impl-Drop-for-Reserve%3C'h,+K,+V,+H%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'h, K, V, H&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"scc/hash_map/struct.Reserve.html\" title=\"struct scc::hash_map::Reserve\">Reserve</a>&lt;'h, K, V, H&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.drop\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2226-2233\">source</a><a href=\"#method.drop\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\" class=\"fn\">drop</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Executes the destructor for this type. <a href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html#tymethod.drop\">Read more</a></div></details></div></details>","Drop","scc::hash_set::Reserve"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Reserve%3C'h,+K,+V,+H%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2172-2183\">source</a><a href=\"#impl-Reserve%3C'h,+K,+V,+H%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'h, K, V, H&gt; <a class=\"struct\" href=\"scc/hash_map/struct.Reserve.html\" title=\"struct scc::hash_map::Reserve\">Reserve</a>&lt;'h, K, V, H&gt;<div class=\"where\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.BuildHasher.html\" title=\"trait core::hash::BuildHasher\">BuildHasher</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.additional_capacity\" class=\"method\"><a class=\"src rightside\" href=\"src/scc/hash_map.rs.html#2180-2182\">source</a><h4 class=\"code-header\">pub fn <a href=\"scc/hash_map/struct.Reserve.html#tymethod.additional_capacity\" class=\"fn\">additional_capacity</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a></h4></section></summary><div class=\"docblock\"><p>Returns the number of reserved slots.</p>\n</div></details></div></details>",0,"scc::hash_set::Reserve"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
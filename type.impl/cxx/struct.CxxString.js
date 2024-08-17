(function() {var type_impls = {
"cxx":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#91-208\">source</a><a href=\"#impl-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#94-96\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.new\" class=\"fn\">new</a>&lt;T: Private&gt;() -&gt; Self</h4></section></summary><div class=\"docblock\"><p><code>CxxString</code> is not constructible via <code>new</code>. Instead, use the\n<a href=\"cxx/macro.let_cxx_string.html\" title=\"macro cxx::let_cxx_string\"><code>let_cxx_string!</code></a> macro.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.len\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#103-105\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.len\" class=\"fn\">len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a></h4></section></summary><div class=\"docblock\"><p>Returns the length of the string in bytes.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/string/basic_string/size\">std::string::size</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_empty\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#112-114\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.is_empty\" class=\"fn\">is_empty</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Returns true if <code>self</code> has a length of zero bytes.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/string/basic_string/empty\">std::string::empty</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_bytes\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#117-121\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.as_bytes\" class=\"fn\">as_bytes</a>(&amp;self) -&gt; &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>] <a href=\"#\" class=\"tooltip\" data-notable-ty=\"&amp;[u8]\">ⓘ</a></h4></section></summary><div class=\"docblock\"><p>Returns a byte slice of this string’s contents.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_ptr\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#134-136\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.as_ptr\" class=\"fn\">as_ptr</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.pointer.html\">*const </a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a></h4></section></summary><div class=\"docblock\"><p>Produces a pointer to the first character of the string.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/string/basic_string/data\">std::string::data</a>.</p>\n<p>Note that the return type may look like <code>const char *</code> but is not a\n<code>const char *</code> in the typical C sense, as C++ strings may contain\ninternal null bytes. As such, the returned pointer only makes sense as a\nstring in combination with the length returned by <a href=\"#method.len\"><code>len()</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_str\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#140-142\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.to_str\" class=\"fn\">to_str</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;&amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/str/error/struct.Utf8Error.html\" title=\"struct core::str::error::Utf8Error\">Utf8Error</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Validates that the C++ string contains UTF-8 data and produces a view of\nit as a Rust &amp;str, otherwise an error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_string_lossy\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#152-154\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.to_string_lossy\" class=\"fn\">to_string_lossy</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/alloc/borrow/enum.Cow.html\" title=\"enum alloc::borrow::Cow\">Cow</a>&lt;'_, <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;</h4></section></summary><div class=\"docblock\"><p>If the contents of the C++ string are valid UTF-8, this function returns\na view as a Cow::Borrowed &amp;str. Otherwise replaces any invalid UTF-8\nsequences with the U+FFFD <a href=\"https://doc.rust-lang.org/std/char/constant.REPLACEMENT_CHARACTER.html\">replacement character</a> and returns a\nCow::Owned String.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clear\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#167-169\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.clear\" class=\"fn\">clear</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;)</h4></section></summary><div class=\"docblock\"><p>Removes all characters from the string.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/string/basic_string/clear\">std::string::clear</a>.</p>\n<p>Note: <strong>unlike</strong> the guarantee of Rust’s <code>std::string::String::clear</code>,\nthe C++ standard does not require that capacity is unchanged by this\noperation. In practice existing implementations do not change the\ncapacity but all pointers, references, and iterators into the string\ncontents are nevertheless invalidated.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.reserve\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#191-197\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.reserve\" class=\"fn\">reserve</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;, additional: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>)</h4></section></summary><div class=\"docblock\"><p>Ensures that this string’s capacity is at least <code>additional</code> bytes\nlarger than its length.</p>\n<p>The capacity may be increased by more than <code>additional</code> bytes if it\nchooses, to amortize the cost of frequent reallocations.</p>\n<p><strong>The meaning of the argument is not the same as\n<a href=\"https://en.cppreference.com/w/cpp/string/basic_string/reserve\">std::string::reserve</a> in C++.</strong> The C++ standard library and\nRust standard library both have a <code>reserve</code> method on strings, but in\nC++ code the argument always refers to total capacity, whereas in Rust\ncode it always refers to additional capacity. This API on <code>CxxString</code>\nfollows the Rust convention, the same way that for the length accessor\nwe use the Rust conventional <code>len()</code> naming and not C++ <code>size()</code> or\n<code>length()</code>.</p>\n<h5 id=\"panics\"><a class=\"doc-anchor\" href=\"#panics\">§</a>Panics</h5>\n<p>Panics if the new capacity overflows usize.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.push_str\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#200-202\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.push_str\" class=\"fn\">push_str</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;, s: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>)</h4></section></summary><div class=\"docblock\"><p>Appends a given string slice onto the end of this C++ string.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.push_bytes\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#205-207\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxString.html#tymethod.push_bytes\" class=\"fn\">push_bytes</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;, bytes: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>])</h4></section></summary><div class=\"docblock\"><p>Appends arbitrary bytes onto the end of this C++ string.</p>\n</div></details></div></details>",0,"cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#216-220\">source</a><a href=\"#impl-Debug-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#217-219\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Display-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#210-214\">source</a><a href=\"#impl-Display-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#211-213\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\">Read more</a></div></details></div></details>","Display","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ExternType-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/extern_type.rs.html#203-225\">source</a><a href=\"#impl-ExternType-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/trait.ExternType.html\" title=\"trait cxx::ExternType\">ExternType</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Kind\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Kind\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"cxx/trait.ExternType.html#associatedtype.Kind\" class=\"associatedtype\">Kind</a> = <a class=\"enum\" href=\"cxx/kind/enum.Opaque.html\" title=\"enum cxx::kind::Opaque\">Opaque</a></h4></section></summary><div class='docblock'>Either <a href=\"cxx/kind/enum.Opaque.html\" title=\"enum cxx::kind::Opaque\"><code>cxx::kind::Opaque</code></a> or <a href=\"cxx/kind/enum.Trivial.html\" title=\"enum cxx::kind::Trivial\"><code>cxx::kind::Trivial</code></a>. <a href=\"cxx/trait.ExternType.html#associatedtype.Kind\">Read more</a></div></details><details class=\"toggle\" open><summary><section id=\"associatedtype.Id\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Id\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"cxx/trait.ExternType.html#associatedtype.Id\" class=\"associatedtype\">Id</a></h4></section></summary><div class='docblock'>A type-level representation of the type’s C++ namespace and type name. <a href=\"cxx/trait.ExternType.html#associatedtype.Id\">Read more</a></div></details></div></details>","ExternType","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Hash-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#254-258\">source</a><a href=\"#impl-Hash-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#255-257\">source</a><a href=\"#method.hash\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#tymethod.hash\" class=\"fn\">hash</a>&lt;H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>&gt;(&amp;self, state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut H</a>)</h4></section></summary><div class='docblock'>Feeds this value into the given <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#tymethod.hash\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.hash_slice\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.3.0\">1.3.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/hash/mod.rs.html#235-237\">source</a></span><a href=\"#method.hash_slice\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#method.hash_slice\" class=\"fn\">hash_slice</a>&lt;H&gt;(data: &amp;[Self], state: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut H</a>)<div class=\"where\">where\n    H: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\">Hasher</a>,\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Feeds a slice of this type into the given <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hasher.html\" title=\"trait core::hash::Hasher\"><code>Hasher</code></a>. <a href=\"https://doc.rust-lang.org/nightly/core/hash/trait.Hash.html#method.hash_slice\">Read more</a></div></details></div></details>","Hash","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Ord-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#248-252\">source</a><a href=\"#impl-Ord-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html\" title=\"trait core::cmp::Ord\">Ord</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.cmp\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#249-251\">source</a><a href=\"#method.cmp\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#tymethod.cmp\" class=\"fn\">cmp</a>(&amp;self, other: &amp;Self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/cmp/enum.Ordering.html\" title=\"enum core::cmp::Ordering\">Ordering</a></h4></section></summary><div class='docblock'>This method returns an <a href=\"https://doc.rust-lang.org/nightly/core/cmp/enum.Ordering.html\" title=\"enum core::cmp::Ordering\"><code>Ordering</code></a> between <code>self</code> and <code>other</code>. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#tymethod.cmp\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.max\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.21.0\">1.21.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#854-856\">source</a></span><a href=\"#method.max\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.max\" class=\"fn\">max</a>(self, other: Self) -&gt; Self<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Compares and returns the maximum of two values. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.max\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.min\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.21.0\">1.21.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#875-877\">source</a></span><a href=\"#method.min\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.min\" class=\"fn\">min</a>(self, other: Self) -&gt; Self<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Compares and returns the minimum of two values. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.min\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clamp\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.50.0\">1.50.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#901-904\">source</a></span><a href=\"#method.clamp\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.clamp\" class=\"fn\">clamp</a>(self, min: Self, max: Self) -&gt; Self<div class=\"where\">where\n    Self: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html\" title=\"trait core::cmp::PartialOrd\">PartialOrd</a>,</div></h4></section></summary><div class='docblock'>Restrict a value to a certain interval. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Ord.html#method.clamp\">Read more</a></div></details></div></details>","Ord","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq%3Cstr%3E-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#234-238\">source</a><a href=\"#impl-PartialEq%3Cstr%3E-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt; for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#235-237\">source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>self</code> and <code>other</code> values to be equal, and is used by <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#261\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>!=</code>. The default implementation is almost always sufficient,\nand should not be overridden without very good reason.</div></details></div></details>","PartialEq<str>","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#222-226\">source</a><a href=\"#impl-PartialEq-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#223-225\">source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;Self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>self</code> and <code>other</code> values to be equal, and is used by <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#261\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>!=</code>. The default implementation is almost always sufficient,\nand should not be overridden without very good reason.</div></details></div></details>","PartialEq","cxx::String"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialOrd-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#242-246\">source</a><a href=\"#impl-PartialOrd-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html\" title=\"trait core::cmp::PartialOrd\">PartialOrd</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.partial_cmp\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#243-245\">source</a><a href=\"#method.partial_cmp\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp\" class=\"fn\">partial_cmp</a>(&amp;self, other: &amp;Self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/cmp/enum.Ordering.html\" title=\"enum core::cmp::Ordering\">Ordering</a>&gt;</h4></section></summary><div class='docblock'>This method returns an ordering between <code>self</code> and <code>other</code> values if one exists. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.lt\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#1178\">source</a></span><a href=\"#method.lt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.lt\" class=\"fn\">lt</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests less than (for <code>self</code> and <code>other</code>) and is used by the <code>&lt;</code> operator. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.lt\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.le\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#1196\">source</a></span><a href=\"#method.le\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.le\" class=\"fn\">le</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests less than or equal to (for <code>self</code> and <code>other</code>) and is used by the\n<code>&lt;=</code> operator. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.le\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.gt\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#1214\">source</a></span><a href=\"#method.gt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.gt\" class=\"fn\">gt</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests greater than (for <code>self</code> and <code>other</code>) and is used by the <code>&gt;</code>\noperator. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.gt\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ge\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#1232\">source</a></span><a href=\"#method.ge\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.ge\" class=\"fn\">ge</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests greater than or equal to (for <code>self</code> and <code>other</code>) and is used by\nthe <code>&gt;=</code> operator. <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialOrd.html#method.ge\">Read more</a></div></details></div></details>","PartialOrd","cxx::String"],["<section id=\"impl-Eq-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_string.rs.html#240\">source</a><a href=\"#impl-Eq-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section>","Eq","cxx::String"],["<section id=\"impl-SharedPtrTarget-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/shared_ptr.rs.html#273\">source</a><a href=\"#impl-SharedPtrTarget-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/memory/trait.SharedPtrTarget.html\" title=\"trait cxx::memory::SharedPtrTarget\">SharedPtrTarget</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section>","SharedPtrTarget","cxx::String"],["<section id=\"impl-UniquePtrTarget-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/unique_ptr.rs.html#282-307\">source</a><a href=\"#impl-UniquePtrTarget-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/memory/trait.UniquePtrTarget.html\" title=\"trait cxx::memory::UniquePtrTarget\">UniquePtrTarget</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section>","UniquePtrTarget","cxx::String"],["<section id=\"impl-VectorElement-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#495\">source</a><a href=\"#impl-VectorElement-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/vector/trait.VectorElement.html\" title=\"trait cxx::vector::VectorElement\">VectorElement</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section>","VectorElement","cxx::String"],["<section id=\"impl-WeakPtrTarget-for-CxxString\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/weak_ptr.rs.html#179\">source</a><a href=\"#impl-WeakPtrTarget-for-CxxString\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"cxx/memory/trait.WeakPtrTarget.html\" title=\"trait cxx::memory::WeakPtrTarget\">WeakPtrTarget</a> for <a class=\"struct\" href=\"cxx/struct.CxxString.html\" title=\"struct cxx::CxxString\">CxxString</a></h3></section>","WeakPtrTarget","cxx::String"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
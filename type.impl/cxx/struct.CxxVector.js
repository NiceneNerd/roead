(function() {var type_impls = {
"cxx":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CxxVector%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#36-199\">source</a><a href=\"#impl-CxxVector%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"struct\" href=\"cxx/struct.CxxVector.html\" title=\"struct cxx::CxxVector\">CxxVector</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/vector/trait.VectorElement.html\" title=\"trait cxx::vector::VectorElement\">VectorElement</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#43-45\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.new\" class=\"fn\">new</a>() -&gt; <a class=\"struct\" href=\"cxx/struct.UniquePtr.html\" title=\"struct cxx::UniquePtr\">UniquePtr</a>&lt;Self&gt; <a href=\"#\" class=\"tooltip\" data-notable-ty=\"UniquePtr&lt;Self&gt;\">ⓘ</a></h4></section></summary><div class=\"docblock\"><p>Constructs a new heap allocated vector, wrapped by UniquePtr.</p>\n<p>The C++ vector is default constructed.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.len\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#52-54\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.len\" class=\"fn\">len</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a></h4></section></summary><div class=\"docblock\"><p>Returns the number of elements in the vector.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/container/vector/size\">std::vector&lt;T&gt;::size</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_empty\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#61-63\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.is_empty\" class=\"fn\">is_empty</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Returns true if the vector contains no elements.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/container/vector/empty\">std::vector&lt;T&gt;::empty</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#67-73\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.get\" class=\"fn\">get</a>(&amp;self, pos: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;T</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Returns a reference to an element at the given position, or <code>None</code> if\nout of bounds.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.index_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#77-83\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.index_mut\" class=\"fn\">index_mut</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;, pos: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut T</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Returns a pinned mutable reference to an element at the given position,\nor <code>None</code> if out of bounds.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_unchecked\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#95-101\">source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"cxx/struct.CxxVector.html#tymethod.get_unchecked\" class=\"fn\">get_unchecked</a>(&amp;self, pos: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;T</a></h4></section></summary><div class=\"docblock\"><p>Returns a reference to an element without doing bounds checking.</p>\n<p>This is generally not recommended, use with caution! Calling this method\nwith an out-of-bounds index is undefined behavior even if the resulting\nreference is not used.</p>\n<p>Matches the behavior of C++\n<a href=\"https://en.cppreference.com/w/cpp/container/vector/operator_at\">std::vector&lt;T&gt;::operator[] const</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.index_unchecked_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#114-119\">source</a><h4 class=\"code-header\">pub unsafe fn <a href=\"cxx/struct.CxxVector.html#tymethod.index_unchecked_mut\" class=\"fn\">index_unchecked_mut</a>(\n    self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;,\n    pos: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>,\n) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut T</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Returns a pinned mutable reference to an element without doing bounds\nchecking.</p>\n<p>This is generally not recommended, use with caution! Calling this method\nwith an out-of-bounds index is undefined behavior even if the resulting\nreference is not used.</p>\n<p>Matches the behavior of C++\n<a href=\"https://en.cppreference.com/w/cpp/container/vector/operator_at\">std::vector&lt;T&gt;::operator[]</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_slice\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#122-140\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.as_slice\" class=\"fn\">as_slice</a>(&amp;self) -&gt; &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">[T]</a><div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/trait.ExternType.html\" title=\"trait cxx::ExternType\">ExternType</a>&lt;Kind = <a class=\"enum\" href=\"cxx/kind/enum.Trivial.html\" title=\"enum cxx::kind::Trivial\">Trivial</a>&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Returns a slice to the underlying contiguous array of elements.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_mut_slice\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#144-155\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.as_mut_slice\" class=\"fn\">as_mut_slice</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;) -&gt; &amp;mut <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">[T]</a><div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/trait.ExternType.html\" title=\"trait cxx::ExternType\">ExternType</a>&lt;Kind = <a class=\"enum\" href=\"cxx/kind/enum.Trivial.html\" title=\"enum cxx::kind::Trivial\">Trivial</a>&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Returns a slice to the underlying contiguous array of elements by\nmutable reference.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.iter\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#158-160\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.iter\" class=\"fn\">iter</a>(&amp;self) -&gt; <a class=\"struct\" href=\"cxx/vector/struct.Iter.html\" title=\"struct cxx::vector::Iter\">Iter</a>&lt;'_, T&gt; <a href=\"#\" class=\"tooltip\" data-notable-ty=\"Iter&lt;&#39;_, T&gt;\">ⓘ</a></h4></section></summary><div class=\"docblock\"><p>Returns an iterator over elements of type <code>&amp;T</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.iter_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#163-165\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.iter_mut\" class=\"fn\">iter_mut</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;) -&gt; <a class=\"struct\" href=\"cxx/vector/struct.IterMut.html\" title=\"struct cxx::vector::IterMut\">IterMut</a>&lt;'_, T&gt; <a href=\"#\" class=\"tooltip\" data-notable-ty=\"IterMut&lt;&#39;_, T&gt;\">ⓘ</a></h4></section></summary><div class=\"docblock\"><p>Returns an iterator over elements of type <code>Pin&lt;&amp;mut T&gt;</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.push\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#172-181\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.push\" class=\"fn\">push</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;, value: T)<div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/trait.ExternType.html\" title=\"trait cxx::ExternType\">ExternType</a>&lt;Kind = <a class=\"enum\" href=\"cxx/kind/enum.Trivial.html\" title=\"enum cxx::kind::Trivial\">Trivial</a>&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Appends an element to the back of the vector.</p>\n<p>Matches the behavior of C++ <a href=\"https://en.cppreference.com/w/cpp/container/vector/push_back\">std::vector&lt;T&gt;::push_back</a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.pop\" class=\"method\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#185-198\">source</a><h4 class=\"code-header\">pub fn <a href=\"cxx/struct.CxxVector.html#tymethod.pop\" class=\"fn\">pop</a>(self: <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/pin/struct.Pin.html\" title=\"struct core::pin::Pin\">Pin</a>&lt;&amp;mut Self&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/trait.ExternType.html\" title=\"trait cxx::ExternType\">ExternType</a>&lt;Kind = <a class=\"enum\" href=\"cxx/kind/enum.Trivial.html\" title=\"enum cxx::kind::Trivial\">Trivial</a>&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Removes the last element from a vector and returns it, or <code>None</code> if the\nvector is empty.</p>\n</div></details></div></details>",0,"cxx::Vector"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-CxxVector%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#304-311\">source</a><a href=\"#impl-Debug-for-CxxVector%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"cxx/struct.CxxVector.html\" title=\"struct cxx::CxxVector\">CxxVector</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/vector/trait.VectorElement.html\" title=\"trait cxx::vector::VectorElement\">VectorElement</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/cxx/cxx_vector.rs.html#308-310\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, formatter: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","cxx::Vector"],["<section id=\"impl-UniquePtrTarget-for-CxxVector%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/cxx/unique_ptr.rs.html#309-331\">source</a><a href=\"#impl-UniquePtrTarget-for-CxxVector%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"cxx/memory/trait.UniquePtrTarget.html\" title=\"trait cxx::memory::UniquePtrTarget\">UniquePtrTarget</a> for <a class=\"struct\" href=\"cxx/struct.CxxVector.html\" title=\"struct cxx::CxxVector\">CxxVector</a>&lt;T&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"cxx/vector/trait.VectorElement.html\" title=\"trait cxx::vector::VectorElement\">VectorElement</a>,</div></h3></section>","UniquePtrTarget","cxx::Vector"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
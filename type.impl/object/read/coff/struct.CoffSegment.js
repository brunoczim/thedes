(function() {var type_impls = {
"object":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-CoffSegment%3C'data,+'file,+R,+Coff%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#148\">source</a><a href=\"#impl-Debug-for-CoffSegment%3C'data,+'file,+R,+Coff%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'data, 'file, R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"object/read/trait.ReadRef.html\" title=\"trait object::read::ReadRef\">ReadRef</a>&lt;'data&gt;, Coff: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"object/read/coff/trait.CoffHeader.html\" title=\"trait object::read::coff::CoffHeader\">CoffHeader</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"object/read/coff/struct.CoffSegment.html\" title=\"struct object::read::coff::CoffSegment\">CoffSegment</a>&lt;'data, 'file, R, Coff&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#148\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","object::read::coff::section::CoffBigSegment"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ObjectSegment%3C'data%3E-for-CoffSegment%3C'data,+'file,+R,+Coff%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#172-230\">source</a><a href=\"#impl-ObjectSegment%3C'data%3E-for-CoffSegment%3C'data,+'file,+R,+Coff%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'data, 'file, R: <a class=\"trait\" href=\"object/read/trait.ReadRef.html\" title=\"trait object::read::ReadRef\">ReadRef</a>&lt;'data&gt;, Coff: <a class=\"trait\" href=\"object/read/coff/trait.CoffHeader.html\" title=\"trait object::read::coff::CoffHeader\">CoffHeader</a>&gt; <a class=\"trait\" href=\"object/read/trait.ObjectSegment.html\" title=\"trait object::read::ObjectSegment\">ObjectSegment</a>&lt;'data&gt; for <a class=\"struct\" href=\"object/read/coff/struct.CoffSegment.html\" title=\"struct object::read::coff::CoffSegment\">CoffSegment</a>&lt;'data, 'file, R, Coff&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.address\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#176-178\">source</a><a href=\"#method.address\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.address\" class=\"fn\">address</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Returns the virtual address of the segment.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.size\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#181-183\">source</a><a href=\"#method.size\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.size\" class=\"fn\">size</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Returns the size of the segment in memory.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.align\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#186-188\">source</a><a href=\"#method.align\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.align\" class=\"fn\">align</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Returns the alignment of the segment in memory.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.file_range\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#191-194\">source</a><a href=\"#method.file_range\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.file_range\" class=\"fn\">file_range</a>(&amp;self) -&gt; (<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a>, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a>)</h4></section></summary><div class='docblock'>Returns the offset and size of the segment in the file.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.data\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#196-198\">source</a><a href=\"#method.data\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.data\" class=\"fn\">data</a>(&amp;self) -&gt; <a class=\"type\" href=\"object/read/type.Result.html\" title=\"type object::read::Result\">Result</a>&lt;&amp;'data [<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u8.html\">u8</a>]&gt;</h4></section></summary><div class='docblock'>Returns a reference to the file contents of the segment. <a href=\"object/read/trait.ObjectSegment.html#tymethod.data\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.data_range\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#200-207\">source</a><a href=\"#method.data_range\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.data_range\" class=\"fn\">data_range</a>(&amp;self, address: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a>, size: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u64.html\">u64</a>) -&gt; <a class=\"type\" href=\"object/read/type.Result.html\" title=\"type object::read::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;'data [<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u8.html\">u8</a>]&gt;&gt;</h4></section></summary><div class='docblock'>Return the segment data in the given range. <a href=\"object/read/trait.ObjectSegment.html#tymethod.data_range\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.name_bytes\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#210-214\">source</a><a href=\"#method.name_bytes\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.name_bytes\" class=\"fn\">name_bytes</a>(&amp;self) -&gt; <a class=\"type\" href=\"object/read/type.Result.html\" title=\"type object::read::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.u8.html\">u8</a>]&gt;&gt;</h4></section></summary><div class='docblock'>Returns the name of the segment.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.name\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#217-223\">source</a><a href=\"#method.name\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.name\" class=\"fn\">name</a>(&amp;self) -&gt; <a class=\"type\" href=\"object/read/type.Result.html\" title=\"type object::read::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.str.html\">str</a>&gt;&gt;</h4></section></summary><div class='docblock'>Returns the name of the segment. <a href=\"object/read/trait.ObjectSegment.html#tymethod.name\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.flags\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/object/read/coff/section.rs.html#226-229\">source</a><a href=\"#method.flags\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"object/read/trait.ObjectSegment.html#tymethod.flags\" class=\"fn\">flags</a>(&amp;self) -&gt; <a class=\"enum\" href=\"object/enum.SegmentFlags.html\" title=\"enum object::SegmentFlags\">SegmentFlags</a></h4></section></summary><div class='docblock'>Return the flags of segment.</div></details></div></details>","ObjectSegment<'data>","object::read::coff::section::CoffBigSegment"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
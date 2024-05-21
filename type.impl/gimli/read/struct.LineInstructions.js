(function() {var type_impls = {
"gimli":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-LineInstructions%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#590\">source</a><a href=\"#impl-Clone-for-LineInstructions%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"gimli/read/trait.Reader.html\" title=\"trait gimli::read::Reader\">Reader</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"gimli/read/struct.LineInstructions.html\" title=\"struct gimli::read::LineInstructions\">LineInstructions</a>&lt;R&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#590\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"gimli/read/struct.LineInstructions.html\" title=\"struct gimli::read::LineInstructions\">LineInstructions</a>&lt;R&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.78.0/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/core/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.78.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","gimli::read::line::OpcodesIter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-LineInstructions%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#590\">source</a><a href=\"#impl-Debug-for-LineInstructions%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"gimli/read/trait.Reader.html\" title=\"trait gimli::read::Reader\">Reader</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"gimli/read/struct.LineInstructions.html\" title=\"struct gimli::read::LineInstructions\">LineInstructions</a>&lt;R&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#590\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","gimli::read::line::OpcodesIter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-LineInstructions%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#604-632\">source</a><a href=\"#impl-LineInstructions%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R: <a class=\"trait\" href=\"gimli/read/trait.Reader.html\" title=\"trait gimli::read::Reader\">Reader</a>&gt; <a class=\"struct\" href=\"gimli/read/struct.LineInstructions.html\" title=\"struct gimli::read::LineInstructions\">LineInstructions</a>&lt;R&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.next_instruction\" class=\"method\"><a class=\"src rightside\" href=\"src/gimli/read/line.rs.html#616-631\">source</a><h4 class=\"code-header\">pub fn <a href=\"gimli/read/struct.LineInstructions.html#tymethod.next_instruction\" class=\"fn\">next_instruction</a>(\n    &amp;mut self,\n    header: &amp;<a class=\"struct\" href=\"gimli/read/struct.LineProgramHeader.html\" title=\"struct gimli::read::LineProgramHeader\">LineProgramHeader</a>&lt;R&gt;\n) -&gt; <a class=\"type\" href=\"gimli/read/type.Result.html\" title=\"type gimli::read::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"gimli/read/enum.LineInstruction.html\" title=\"enum gimli::read::LineInstruction\">LineInstruction</a>&lt;R&gt;&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Advance the iterator and return the next instruction.</p>\n<p>Returns the newly parsed instruction as <code>Ok(Some(instruction))</code>. Returns\n<code>Ok(None)</code> when iteration is complete and all instructions have already been\nparsed and yielded. If an error occurs while parsing the next attribute,\nthen this error is returned as <code>Err(e)</code>, and all subsequent calls return\n<code>Ok(None)</code>.</p>\n<p>Unfortunately, the <code>header</code> parameter means that this cannot be a\n<code>FallibleIterator</code>.</p>\n</div></details></div></details>",0,"gimli::read::line::OpcodesIter"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
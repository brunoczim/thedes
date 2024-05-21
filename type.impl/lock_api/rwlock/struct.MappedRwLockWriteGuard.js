(function() {var type_impls = {
"parking_lot":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2866-2867\">source</a><a href=\"#impl-Debug-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2869\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deref-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2841\">source</a><a href=\"#impl-Deref-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Target\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Target\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/1.78.0/core/ops/deref/trait.Deref.html#associatedtype.Target\" class=\"associatedtype\">Target</a> = T</h4></section></summary><div class='docblock'>The resulting type after dereferencing.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2844\">source</a><a href=\"#method.deref\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/ops/deref/trait.Deref.html#tymethod.deref\" class=\"fn\">deref</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;T</a></h4></section></summary><div class='docblock'>Dereferences the value.</div></details></div></details>","Deref","parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-DerefMut-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2849\">source</a><a href=\"#impl-DerefMut-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/ops/deref/trait.DerefMut.html\" title=\"trait core::ops::deref::DerefMut\">DerefMut</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.deref_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2851\">source</a><a href=\"#method.deref_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/ops/deref/trait.DerefMut.html#tymethod.deref_mut\" class=\"fn\">deref_mut</a>(&amp;mut self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;mut T</a></h4></section></summary><div class='docblock'>Mutably dereferences the value.</div></details></div></details>","DerefMut","parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Display-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2874-2875\">source</a><a href=\"#impl-Display-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> + 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2877\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Display.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.78.0/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.78.0/core/fmt/trait.Display.html#tymethod.fmt\">Read more</a></div></details></div></details>","Display","parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Drop-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2856\">source</a><a href=\"#impl-Drop-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.drop\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2858\">source</a><a href=\"#method.drop\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.78.0/core/ops/drop/trait.Drop.html#tymethod.drop\" class=\"fn\">drop</a>(&amp;mut self)</h4></section></summary><div class='docblock'>Executes the destructor for this type. <a href=\"https://doc.rust-lang.org/1.78.0/core/ops/drop/trait.Drop.html#tymethod.drop\">Read more</a></div></details></div></details>","Drop","parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2766\">source</a><a href=\"#impl-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.map\" class=\"method\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2776-2778\">source</a><h4 class=\"code-header\">pub fn <a href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html#tymethod.map\" class=\"fn\">map</a>&lt;U, F&gt;(\n    s: <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;,\n    f: F\n) -&gt; <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, U&gt;<div class=\"where\">where\n    F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;mut T</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;mut U</a>,\n    U: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class=\"docblock\"><p>Make a new <code>MappedRwLockWriteGuard</code> for a component of the locked data.</p>\n<p>This operation cannot fail as the <code>MappedRwLockWriteGuard</code> passed\nin already locked the data.</p>\n<p>This is an associated function that needs to be\nused as <code>MappedRwLockWriteGuard::map(...)</code>. A method would interfere with methods of\nthe same name on the contents of the locked data.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.try_map\" class=\"method\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2800-2802\">source</a><h4 class=\"code-header\">pub fn <a href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html#tymethod.try_map\" class=\"fn\">try_map</a>&lt;U, F&gt;(\n    s: <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;,\n    f: F\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, U&gt;, <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;&gt;<div class=\"where\">where\n    F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;mut T</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.78.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.78.0/std/primitive.reference.html\">&amp;mut U</a>&gt;,\n    U: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class=\"docblock\"><p>Attempts to make  a new <code>MappedRwLockWriteGuard</code> for a component of the\nlocked data. The original guard is return if the closure returns <code>None</code>.</p>\n<p>This operation cannot fail as the <code>MappedRwLockWriteGuard</code> passed\nin already locked the data.</p>\n<p>This is an associated function that needs to be\nused as <code>MappedRwLockWriteGuard::try_map(...)</code>. A method would interfere with methods of\nthe same name on the contents of the locked data.</p>\n</div></details></div></details>",0,"parking_lot::rwlock::MappedRwLockWriteGuard"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2818\">source</a><a href=\"#impl-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLockFair.html\" title=\"trait lock_api::rwlock::RawRwLockFair\">RawRwLockFair</a> + 'a,\n    T: 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.unlock_fair\" class=\"method\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2832\">source</a><h4 class=\"code-header\">pub fn <a href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html#tymethod.unlock_fair\" class=\"fn\">unlock_fair</a>(s: <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;)</h4></section></summary><div class=\"docblock\"><p>Unlocks the <code>RwLock</code> using a fair unlock protocol.</p>\n<p>By default, <code>RwLock</code> is unfair and allow the current thread to re-lock\nthe <code>RwLock</code> before another has the chance to acquire the lock, even if\nthat thread has been blocked on the <code>RwLock</code> for a long time. This is\nthe default because it allows much higher throughput as it avoids\nforcing a context switch on every <code>RwLock</code> unlock. This can result in one\nthread acquiring a <code>RwLock</code> many more times than other threads.</p>\n<p>However in some cases it can be beneficial to ensure fairness by forcing\nthe lock to pass on to a waiting thread if there is one. This is done by\nusing this method instead of dropping the <code>MappedRwLockWriteGuard</code> normally.</p>\n</div></details></div></details>",0,"parking_lot::rwlock::MappedRwLockWriteGuard"],["<section id=\"impl-Send-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2761-2762\">source</a><a href=\"#impl-Send-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,\n    &lt;R as <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a>&gt;::<a class=\"associatedtype\" href=\"lock_api/rwlock/trait.RawRwLock.html#associatedtype.GuardMarker\" title=\"type lock_api::rwlock::RawRwLock::GuardMarker\">GuardMarker</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,</div></h3></section>","Send","parking_lot::rwlock::MappedRwLockWriteGuard"],["<section id=\"impl-Sync-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/lock_api/rwlock.rs.html#2757-2758\">source</a><a href=\"#impl-Sync-for-MappedRwLockWriteGuard%3C'a,+R,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, R, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"lock_api/rwlock/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::rwlock::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"lock_api/rwlock/trait.RawRwLock.html\" title=\"trait lock_api::rwlock::RawRwLock\">RawRwLock</a> + 'a,\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'a + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.78.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h3></section>","Sync","parking_lot::rwlock::MappedRwLockWriteGuard"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()
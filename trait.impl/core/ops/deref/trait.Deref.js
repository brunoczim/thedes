(function() {
    var implementors = Object.fromEntries([["lock_api",[["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawMutex.html\" title=\"trait lock_api::RawMutex\">RawMutex</a> + 'a, G: <a class=\"trait\" href=\"lock_api/trait.GetThreadId.html\" title=\"trait lock_api::GetThreadId\">GetThreadId</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.MappedReentrantMutexGuard.html\" title=\"struct lock_api::MappedReentrantMutexGuard\">MappedReentrantMutexGuard</a>&lt;'a, R, G, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawMutex.html\" title=\"trait lock_api::RawMutex\">RawMutex</a> + 'a, G: <a class=\"trait\" href=\"lock_api/trait.GetThreadId.html\" title=\"trait lock_api::GetThreadId\">GetThreadId</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.ReentrantMutexGuard.html\" title=\"struct lock_api::ReentrantMutexGuard\">ReentrantMutexGuard</a>&lt;'a, R, G, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawMutex.html\" title=\"trait lock_api::RawMutex\">RawMutex</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.MappedMutexGuard.html\" title=\"struct lock_api::MappedMutexGuard\">MappedMutexGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawMutex.html\" title=\"trait lock_api::RawMutex\">RawMutex</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.MutexGuard.html\" title=\"struct lock_api::MutexGuard\">MutexGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawRwLock.html\" title=\"trait lock_api::RawRwLock\">RawRwLock</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.MappedRwLockReadGuard.html\" title=\"struct lock_api::MappedRwLockReadGuard\">MappedRwLockReadGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawRwLock.html\" title=\"trait lock_api::RawRwLock\">RawRwLock</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.MappedRwLockWriteGuard.html\" title=\"struct lock_api::MappedRwLockWriteGuard\">MappedRwLockWriteGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawRwLock.html\" title=\"trait lock_api::RawRwLock\">RawRwLock</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.RwLockReadGuard.html\" title=\"struct lock_api::RwLockReadGuard\">RwLockReadGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawRwLock.html\" title=\"trait lock_api::RawRwLock\">RawRwLock</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.RwLockWriteGuard.html\" title=\"struct lock_api::RwLockWriteGuard\">RwLockWriteGuard</a>&lt;'a, R, T&gt;"],["impl&lt;'a, R: <a class=\"trait\" href=\"lock_api/trait.RawRwLockUpgrade.html\" title=\"trait lock_api::RawRwLockUpgrade\">RawRwLockUpgrade</a> + 'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a> + 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"lock_api/struct.RwLockUpgradableReadGuard.html\" title=\"struct lock_api::RwLockUpgradableReadGuard\">RwLockUpgradableReadGuard</a>&lt;'a, R, T&gt;"]]],["once_cell",[["impl&lt;T, F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"once_cell/sync/struct.Lazy.html\" title=\"struct once_cell::sync::Lazy\">Lazy</a>&lt;T, F&gt;"],["impl&lt;T, F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"once_cell/unsync/struct.Lazy.html\" title=\"struct once_cell::unsync::Lazy\">Lazy</a>&lt;T, F&gt;"]]],["regex_automata",[["impl&lt;'a, T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>, F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"regex_automata/util/pool/struct.PoolGuard.html\" title=\"struct regex_automata::util::pool::PoolGuard\">PoolGuard</a>&lt;'a, T, F&gt;"],["impl&lt;T, F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"regex_automata/util/lazy/struct.Lazy.html\" title=\"struct regex_automata::util::lazy::Lazy\">Lazy</a>&lt;T, F&gt;"]]],["scopeguard",[["impl&lt;T, F, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"scopeguard/struct.ScopeGuard.html\" title=\"struct scopeguard::ScopeGuard\">ScopeGuard</a>&lt;T, F, S&gt;<div class=\"where\">where\n    F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>(T),\n    S: <a class=\"trait\" href=\"scopeguard/trait.Strategy.html\" title=\"trait scopeguard::Strategy\">Strategy</a>,</div>"]]],["sharded_slab",[["impl&lt;'a, T, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/pool/struct.Ref.html\" title=\"struct sharded_slab::pool::Ref\">Ref</a>&lt;'a, T, C&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"sharded_slab/trait.Clear.html\" title=\"trait sharded_slab::Clear\">Clear</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>,</div>"],["impl&lt;'a, T, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/pool/struct.RefMut.html\" title=\"struct sharded_slab::pool::RefMut\">RefMut</a>&lt;'a, T, C&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"sharded_slab/trait.Clear.html\" title=\"trait sharded_slab::Clear\">Clear</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>,</div>"],["impl&lt;'a, T, C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/struct.Entry.html\" title=\"struct sharded_slab::Entry\">Entry</a>&lt;'a, T, C&gt;"],["impl&lt;T, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/pool/struct.OwnedRef.html\" title=\"struct sharded_slab::pool::OwnedRef\">OwnedRef</a>&lt;T, C&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"sharded_slab/trait.Clear.html\" title=\"trait sharded_slab::Clear\">Clear</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>,</div>"],["impl&lt;T, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/pool/struct.OwnedRefMut.html\" title=\"struct sharded_slab::pool::OwnedRefMut\">OwnedRefMut</a>&lt;T, C&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"sharded_slab/trait.Clear.html\" title=\"trait sharded_slab::Clear\">Clear</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>,</div>"],["impl&lt;T, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"sharded_slab/struct.OwnedEntry.html\" title=\"struct sharded_slab::OwnedEntry\">OwnedEntry</a>&lt;T, C&gt;<div class=\"where\">where\n    C: <a class=\"trait\" href=\"sharded_slab/trait.Config.html\" title=\"trait sharded_slab::Config\">Config</a>,</div>"]]],["smallstr",[["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&lt;Item = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.83.0/core/primitive.u8.html\">u8</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smallstr/struct.SmallString.html\" title=\"struct smallstr::SmallString\">SmallString</a>&lt;A&gt;"]]],["smallvec",[["impl&lt;A: <a class=\"trait\" href=\"smallvec/trait.Array.html\" title=\"trait smallvec::Array\">Array</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smallvec/struct.SmallVec.html\" title=\"struct smallvec::SmallVec\">SmallVec</a>&lt;A&gt;"]]],["syn",[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.And.html\" title=\"struct syn::token::And\">And</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.At.html\" title=\"struct syn::token::At\">At</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Caret.html\" title=\"struct syn::token::Caret\">Caret</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Colon.html\" title=\"struct syn::token::Colon\">Colon</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Comma.html\" title=\"struct syn::token::Comma\">Comma</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Dollar.html\" title=\"struct syn::token::Dollar\">Dollar</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Dot.html\" title=\"struct syn::token::Dot\">Dot</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Eq.html\" title=\"struct syn::token::Eq\">Eq</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Gt.html\" title=\"struct syn::token::Gt\">Gt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Lt.html\" title=\"struct syn::token::Lt\">Lt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Minus.html\" title=\"struct syn::token::Minus\">Minus</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Not.html\" title=\"struct syn::token::Not\">Not</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Or.html\" title=\"struct syn::token::Or\">Or</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Percent.html\" title=\"struct syn::token::Percent\">Percent</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Plus.html\" title=\"struct syn::token::Plus\">Plus</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Pound.html\" title=\"struct syn::token::Pound\">Pound</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Question.html\" title=\"struct syn::token::Question\">Question</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Semi.html\" title=\"struct syn::token::Semi\">Semi</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Slash.html\" title=\"struct syn::token::Slash\">Slash</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Star.html\" title=\"struct syn::token::Star\">Star</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Tilde.html\" title=\"struct syn::token::Tilde\">Tilde</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Underscore.html\" title=\"struct syn::token::Underscore\">Underscore</a>"],["impl&lt;'c, 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/parse/struct.StepCursor.html\" title=\"struct syn::parse::StepCursor\">StepCursor</a>&lt;'c, 'a&gt;"]]],["tracing",[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"tracing/span/struct.EnteredSpan.html\" title=\"struct tracing::span::EnteredSpan\">EnteredSpan</a>"]]],["tracing_subscriber",[["impl&lt;E: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"tracing_subscriber/fmt/struct.FormattedFields.html\" title=\"struct tracing_subscriber::fmt::FormattedFields\">FormattedFields</a>&lt;E&gt;"]]],["zerocopy",[["impl&lt;B, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"zerocopy/struct.Ref.html\" title=\"struct zerocopy::Ref\">Ref</a>&lt;B, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.83.0/core/primitive.slice.html\">[T]</a>&gt;<div class=\"where\">where\n    B: <a class=\"trait\" href=\"zerocopy/trait.ByteSlice.html\" title=\"trait zerocopy::ByteSlice\">ByteSlice</a>,\n    T: <a class=\"trait\" href=\"zerocopy/trait.FromBytes.html\" title=\"trait zerocopy::FromBytes\">FromBytes</a>,</div>"],["impl&lt;B, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"zerocopy/struct.Ref.html\" title=\"struct zerocopy::Ref\">Ref</a>&lt;B, T&gt;<div class=\"where\">where\n    B: <a class=\"trait\" href=\"zerocopy/trait.ByteSlice.html\" title=\"trait zerocopy::ByteSlice\">ByteSlice</a>,\n    T: <a class=\"trait\" href=\"zerocopy/trait.FromBytes.html\" title=\"trait zerocopy::FromBytes\">FromBytes</a>,</div>"],["impl&lt;T: <a class=\"trait\" href=\"zerocopy/trait.Unaligned.html\" title=\"trait zerocopy::Unaligned\">Unaligned</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.83.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"zerocopy/struct.Unalign.html\" title=\"struct zerocopy::Unalign\">Unalign</a>&lt;T&gt;"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[5543,924,1100,635,3732,512,394,5948,296,495,1578]}
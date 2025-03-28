(function() {
    var implementors = Object.fromEntries([["futures_sink",[]],["tokio_util",[["impl&lt;L, R, Item, Error&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;Item&gt; for <a class=\"enum\" href=\"tokio_util/either/enum.Either.html\" title=\"enum tokio_util::either::Either\">Either</a>&lt;L, R&gt;<div class=\"where\">where\n    L: <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;Item, Error = Error&gt;,\n    R: <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;Item, Error = Error&gt;,</div>"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.85.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;T&gt; for <a class=\"struct\" href=\"tokio_util/sync/struct.PollSender.html\" title=\"struct tokio_util::sync::PollSender\">PollSender</a>&lt;T&gt;"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[19,1005]}
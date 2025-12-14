(function() {
    var type_impls = Object.fromEntries([["serde",[]],["serde_core",[]],["thedes_domain",[]],["thedes_tui_core",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[12,18,21,23]}
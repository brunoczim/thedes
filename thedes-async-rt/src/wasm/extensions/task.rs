pub fn detach<A>(future: A)
where
    A: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

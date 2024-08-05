fn main() {
    #[cfg(all(feature = "std", not(feature = "metadata-hash")))]
    {
        substrate_wasm_builder::WasmBuilder::new()
            .with_current_project()
            .export_heap_base()
            .import_memory()
            .build();
    }
    #[cfg(all(feature = "std", feature = "metadata-hash"))]
    {
        substrate_wasm_builder::WasmBuilder::new()
            .with_current_project()
            .export_heap_base()
            .import_memory()
            .enable_metadata_hash("TAO", 9)
            .build();
    }
}

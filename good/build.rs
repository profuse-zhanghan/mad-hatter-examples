fn main() {
    mad_hatter_guardian::activate(mad_hatter_guardian::DefenseConfig {
        source_dir: "src",
        concept_map: "concept-map.json",
        projection_file: "src/concept_map.rs",
    });
}
fn main() {
    uniffi::generate_scaffolding("./src/shared.udl")
        .expect("Couldn't generate Swift and Kotlin scaffolding");
}

fn main() {
    // Compile cubiomes C sources
    cc::Build::new()
        .file("cubiomes/biomenoise.c")
        .file("cubiomes/biomes.c")
        .file("cubiomes/finders.c")
        .file("cubiomes/generator.c")
        .file("cubiomes/layers.c")
        .file("cubiomes/noise.c")
        .file("cubiomes/util.c")
        .file("cubiomes_helper.c")
        .include("cubiomes")
        .opt_level(3)
        .warnings(false)
        .compile("cubiomes");

    println!("cargo:rerun-if-changed=cubiomes/");
    println!("cargo:rerun-if-changed=cubiomes_helper.c");
}

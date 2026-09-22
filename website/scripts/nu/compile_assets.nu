def main [
    scripts_dir: string,
    assets_dir: string,
    styles_dir: string,
    output_dir: string
] {
    # Run the asset compression script
    let compress_script = $scripts_dir | path join "compress_assets.nu"
    if not ($compress_script | path exists) {
        error make {msg: $"Compression script '($compress_script)' does not exist."}
    }
    if (nu $compress_script $assets_dir $output_dir | complete).exit_code != 0 {
        error make {msg: "Failed to compress assets."}
    }

    # Run the index generation script
    let index_script = $scripts_dir | path join "generate_index.nu"
    if not ($index_script | path exists) {
        error make {msg: $"Index generation script '($index_script)' does not exist."}
    }
    if (nu $index_script $assets_dir ($output_dir | path join "assets" "_assets" "articles") | complete).exit_code != 0 {
        error make {msg: "Failed to generate index."}
    }

    # Copy fonts to the output directory
    let fonts_dir = $styles_dir | path join "fonts" | str replace "\\" "/"
    let styles_output_dir = $output_dir | path join "assets" "fonts"
    if not ($styles_output_dir | path exists) {
        mkdir $styles_output_dir
    }

    glob $"($fonts_dir)/*.woff2" 
    | each {
        |p| cp $p $styles_output_dir
    }
}
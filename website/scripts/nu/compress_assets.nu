#!/usr/bin/env nu
# This script compiles the assets for the project.

def main [assets_dir: string, output_dir: string] {
    # Check if the required tools are installed
    def check_command [cmd: string] {
        try {
            which $cmd | get path.0 | is-not-empty
        } catch {
            false
        }
    }

    if not (check_command "zstd") {
        error make {msg: "Zstd is not installed. Please install it to compile assets."}
    }

    # Check if assets directory exists
    if not ($assets_dir | path exists) {
        error make {msg: $"Assets directory '($assets_dir)' does not exist."}
    }

    # Create output directory if it doesn't exist
    if not ($output_dir | path exists) {
        mkdir $output_dir
    }

    # Clean up existing files
    let assets_output = $output_dir | path join "assets"
    if ($assets_output | path exists) {
        rm -rf $assets_output
    }

    let site_json = $output_dir | path join "site.json"
    if ($site_json | path exists) {
        rm $site_json
    }

    let abs_assets_dir = pwd | path join $assets_dir

    # Create the assets directory structure
    print "Creating directory structure..."
    mkdir $assets_output

    # Process all files
    glob $"($assets_dir)/**/*"
    | each {
        |p| ls -D $p 
        | flatten
        | where type == file
        | each {|file|
            let relative_path = $file.name | path relative-to $abs_assets_dir
            let output_path = $assets_output | path join $relative_path

            if (($file.name | path basename) == ".special") {
                # Skip special files
                return
            }

            # Ensure parent directory exists
            mkdir ($output_path | path dirname)
            
            match ($file.name | path parse | get extension) {
                "json" => {
                    print $"Minifying JSON: ($file.name) -> ($output_path)"
                    open $file.name | to json -r | save -f $output_path
                }
                "md" => {
                    let zst_output = $output_path | str replace ".md" ".zst"
                    print $"Compressing Markdown: ($file.name) -> ($zst_output)"
                    ^zstd -q $file.name -o $zst_output
                }
                _ => {
                    # For other file types, just copy them
                    cp $file.name $output_path
                }
            }
        }

    }

    print "Assets compiled successfully."
}
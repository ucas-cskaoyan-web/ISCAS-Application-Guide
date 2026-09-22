#!/usr/bin/env nu
# This script generates an index file for the assets directory.

def main [assets_dir: string, output_dir: string] {
    # Check if the required tools are installed
    def check_command [cmd: string] {
        try {
            which $cmd | get path.0 | is-not-empty
        } catch {
            false
        }
    }

    # Check if the assets directory exists
    if not ($assets_dir | path exists) {
        error make {msg: $"Assets directory '($assets_dir)' does not exist."}
    }

    # Create output directory if it doesn't exist
    if not ($output_dir | path exists) {
        mkdir $output_dir
    }

    # Read site.json to get site assets directory and articles directory
    let site_json = $assets_dir | path join "site.json"
    if not ($site_json | path exists) {
        error make {msg: $"site.json not found in '($assets_dir)'."}
    }

    let site_config = open $site_json
    let site_assets_dir = $site_config.assets.directory?
    let articles_dir = $site_config.assets.articles?

    if ($site_assets_dir | is-empty) or ($articles_dir | is-empty) {
        error make {msg: "Invalid site.json format. 'assets.directory' or 'assets.articles' is missing."}
    }

    let site_assets_path = $assets_dir | path join $site_assets_dir
    let articles_path = $site_assets_path | path join $articles_dir
    let index_file = $output_dir | path join "index.json"
    let special_index_file = $output_dir | path join "special.json"

    # Check if the site assets and articles directories exist
    if not ($site_assets_path | path exists) {
        error make {msg: $"Assets directory '($site_assets_path)' does not exist."}
    }

    if not ($articles_path | path exists) {
        error make {msg: $"Articles directory '($articles_path)' does not exist."}
    }

    # Generate special articles index
    print "Generating special articles index..."
    let articles_path = $articles_path | str replace -a '\' '/'
    let special_articles = glob $"($articles_path)/**/.special"
    | each {|special_file|
        let article_dir = $special_file | path dirname
        let article_id = $article_dir | path basename
        let meta_file = $article_dir | path join "meta.json"
        let index_md = $article_dir | path join "index.md"
        
        if ($meta_file | path exists) and ($index_md | path exists) {
            let meta = open $meta_file
            print $"Special article '($article_id)' indexed."
            {
                id: $article_id,
                title: $meta.title,
                description: $meta.description
            }
        } else {
            if not ($meta_file | path exists) {
                print $"Warning: meta.json not found for special article '($article_id)'. Skipping."
            }
            if not ($index_md | path exists) {
                print $"Warning: index.md not found for special article '($article_id)'. Skipping."
            }
            null
        }
    }
    | compact
    | reduce -f {} {|article, acc|
        $acc | insert $article.id {title: $article.title, description: $article.description}
    }

    $special_articles | to json -r | save -f $special_index_file

    # Generate normal articles index
    print "Generating normal articles index..."
    let normal_articles = glob $"($articles_path)/**/index.md"
    | each {|index_file|
        let article_dir = $index_file | path dirname
        let article_id = $article_dir | path basename
        let meta_file = $article_dir | path join "meta.json"
        let special_file = $article_dir | path join ".special"
        
        # Skip if the article is marked as special
        if ($special_file | path exists) {
            null
        } else if ($meta_file | path exists) {
            let meta = open $meta_file
            print $"Article '($article_id)' indexed."
            {
                id: $article_id,
                meta: $meta
            }
        } else {
            print $"Warning: meta.json not found for article '($article_id)'. Skipping."
            null
        }
    }
    | compact
    | reduce -f {} {|article, acc|
        $acc | insert $article.id $article.meta
    }

    $normal_articles | to json -r | save -f $index_file

    print "Index files generated successfully."
}
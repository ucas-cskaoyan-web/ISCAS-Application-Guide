import argparse
import os
import base64
import hashlib
from bs4 import BeautifulSoup

def compute_sha384(filepath):
    hash_obj = hashlib.sha384()
    with open(filepath, 'rb') as f:
        for chunk in iter(lambda: f.read(8192), b''):
            hash_obj.update(chunk)
    digest = hash_obj.digest()
    b64 = base64.b64encode(digest).decode('ascii')
    return f"sha384-{b64}"

def update_integrity(html_path, base_dir=None, inplace=False, url_prefix=None, output=None):
    if base_dir is None:
        base_dir = os.path.dirname(os.path.abspath(html_path))

    if url_prefix.endswith('/'):
        url_prefix = url_prefix[:-1]

    with open(html_path, 'r', encoding='utf-8') as f:
        soup = BeautifulSoup(f, 'html.parser')

    head = soup.head
    if not head:
        print("No <head> section found.")
        return

    # Process all <link> and <script> tags with href/src attributes
    tags = head.find_all(lambda tag: (tag.name == 'link' and tag.has_attr('href')) or 
                                          (tag.name == 'script' and tag.has_attr('src')))
    for tag in tags:
        attr = 'href' if tag.name == 'link' else 'src'
        path = tag[attr]
        # Only process paths that start with '/'
        path_start = url_prefix if url_prefix != None else '/'
        
        if path.startswith(path_start):
            if url_prefix != None:
                path = path[len(path_start):]  # Remove the URL prefix if specified
            local_path = os.path.join(base_dir, path.lstrip('/'))
            if not os.path.isfile(local_path):
                print(f"Warning: file not found {local_path}, skipping.")
                continue
            integrity_value = compute_sha384(local_path)
            tag['integrity'] = integrity_value
            # Add crossorigin if not present
            if not tag.has_attr('crossorigin'):
                tag['crossorigin'] = 'anonymous'
            print(f"Updated {attr}={path} -> integrity={integrity_value}")

    # Output the modified HTML
    out_html = str(soup)
    if inplace:
        with open(html_path, 'w', encoding='utf-8') as f:
            f.write(out_html)
        print(f"File overwritten: {html_path}")
    else:
        tgt = output or html_path + '.updated.html'
        with open(tgt, 'w', encoding='utf-8') as f:
            f.write(out_html)
        print(f"Updated HTML written to {tgt}")

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Update integrity attributes for local resources in HTML head.')
    parser.add_argument('html', help='Path to the HTML file')
    parser.add_argument('-b', '--base-dir', help='Base directory for local files (defaults to HTML directory)')
    parser.add_argument('-i', '--inplace', action='store_true', help='Overwrite the original HTML file')
    parser.add_argument('-u', '--url', help='URL prefix to match local files')
    parser.add_argument('-o', '--output', help='Path for output HTML (if not using inplace)')
    args = parser.parse_args()
    update_integrity(args.html, base_dir=args.base_dir, inplace=args.inplace, url_prefix=args.url, output=args.output)

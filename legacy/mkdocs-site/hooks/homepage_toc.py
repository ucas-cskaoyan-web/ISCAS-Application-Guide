"""Expose the custom homepage's HTML headings to MkDocs plugins."""

from html import escape
from html.parser import HTMLParser

from mkdocs.structure.toc import AnchorLink, TableOfContents


class _HomepageHeadingParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.headings: list[tuple[str, str]] = []
        self._heading_id: str | None = None
        self._heading_text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag != "h2" or self._heading_id is not None:
            return

        heading_id = dict(attrs).get("id")
        if heading_id:
            self._heading_id = heading_id
            self._heading_text = []

    def handle_data(self, data: str) -> None:
        if self._heading_id is not None:
            self._heading_text.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag != "h2" or self._heading_id is None:
            return

        title = " ".join("".join(self._heading_text).split())
        if title:
            self.headings.append((self._heading_id, title))
        self._heading_id = None
        self._heading_text = []


def on_page_content(html: str, *, page, **kwargs) -> str:
    if not page.is_homepage:
        return html

    parser = _HomepageHeadingParser()
    parser.feed(html)
    page.toc = TableOfContents(
        [AnchorLink(escape(title), heading_id, 2) for heading_id, title in parser.headings]
    )
    return html

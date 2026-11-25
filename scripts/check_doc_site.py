"""Check the generated tutorial routes, local link targets and page landmarks."""

from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit

SITE = Path(__file__).resolve().parents[1] / "docs-site" / "dist"
BASE = "/paignia/"
TUTORIALS = (
    "", "start", "static-games", "interchange", "equilibria",
    "normal-form", "information-sets", "applications", "reference",
)


class Page(HTMLParser):
    def __init__(self, path):
        super().__init__()
        self.ids = set()
        self.links = set()
        self.headings = 0
        self.language = ""
        self.skip = False
        self.feed(path.read_text(encoding="utf-8"))

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if attrs.get("id"):
            self.ids.add(attrs["id"])
        if tag == "html":
            self.language = attrs.get("lang", "")
        if tag == "h1":
            self.headings += 1
        url = attrs.get("href") or attrs.get("src")
        if url:
            self.links.add(url)
        if tag == "a" and url == "#_top":
            self.skip |= "skip" in attrs.get("class", "")


def check():
    pages = {path: Page(path) for path in SITE.rglob("*.html")}
    for tutorial in TUTORIALS:
        path = SITE / tutorial / "index.html"
        page = pages.get(path)
        if page is None or not (page.language and page.headings == 1 and page.skip):
            raise ValueError(f"{path}: expected one main heading, language and skip link")

    for path, page in sorted(pages.items()):
        url = BASE + path.relative_to(SITE).as_posix()
        for link in sorted(page.links):
            target = urlsplit(urljoin(url, link))
            if target.scheme or target.netloc:
                continue
            if not target.path.startswith(BASE):
                raise ValueError(f"{url}: {link} leaves the site base")
            local = SITE / unquote(target.path.removeprefix(BASE))
            if local.is_dir():
                local /= "index.html"
            if not local.is_file():
                raise ValueError(f"{url}: missing target {link}")
            if target.fragment and local in pages and unquote(target.fragment) not in pages[local].ids:
                raise ValueError(f"{url}: missing fragment {link}")
    print(f"Verified {len(TUTORIALS)} tutorial routes and links across {len(pages)} pages.")


if __name__ == "__main__":
    check()

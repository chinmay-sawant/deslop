from pathlib import Path

API_TOKEN = "supersecretvalue"

def build_summary(name: str) -> str:
    """Build summary text for the caller."""
    return f"hello {name}"

class Renderer:
    def render(self, path: str) -> str:
        return Path(path).read_text()
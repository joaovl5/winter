from pathlib import Path

from rich import print as p
from rich.highlighter import ReprHighlighter
from rich.panel import Panel
from rich.pretty import Pretty
from rich.rule import Rule

from winter.lexer.lexer import Lexer
from winter.utils.logging import configure_logging

configure_logging()

RICH_HL = ReprHighlighter()
THIS_DIR = Path(__file__).parent
FILE = THIS_DIR / "draft.win"
FILE_CONTENTS = FILE.read_text()
LEXED_FORM_TKS, LEXED_ERRORS = Lexer(iter(FILE_CONTENTS)).parse_total()


p(Rule("# FILE CONTENTS", align="left"))
p(
    Panel(FILE_CONTENTS),
)
p(Rule("# LEXED FORM", align="left"))
p(
    Panel(
        RICH_HL(" | ".join(x.__repr__() for x in LEXED_FORM_TKS)),
    ),
    Panel(
        Pretty(LEXED_ERRORS, indent_size=2),
    ),
)

p(Rule("PARSED FORM"))

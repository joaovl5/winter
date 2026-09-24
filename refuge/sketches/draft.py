import sys
from pathlib import Path

from rich.console import Console
from rich.highlighter import ReprHighlighter
from rich.panel import Panel
from rich.pretty import Pretty
from rich.rule import Rule

from winter.lexer.lexer import Lexer
from winter.parser.parser import Parser
from winter.utils.logging import configure_logging

configure_logging()

RICH_HL = ReprHighlighter()
RICH_CL = Console(
    tab_size=2,
    width=120,
    file=sys.stdout,
)
THIS_DIR = Path(__file__).parent
FILE = THIS_DIR / "draft.win"
FILE_CONTENTS = FILE.read_text()
LEXED_FORM_TKS, LEXED_ERRORS = Lexer(iter(FILE_CONTENTS)).parse_total()
PARSED_FORM_AST, PARSED_ERRORS = Parser(iter(LEXED_FORM_TKS)).parse_total()


def p(*xs: object) -> None:
    RICH_CL.log(*xs)


def _pp(x: object) -> Pretty:
    return Pretty(
        x,
        indent_size=2,
        margin=10,
    )


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
        _pp(LEXED_ERRORS),
    ),
)

p(Rule("PARSED FORM"))

p(
    Panel(
        _pp(PARSED_FORM_AST),
    ),
    Panel(
        _pp(PARSED_ERRORS),
    ),
)

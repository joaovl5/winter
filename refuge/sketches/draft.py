import subprocess
import sys
from pathlib import Path

from rich.console import Console
from rich.highlighter import ReprHighlighter
from rich.pretty import Pretty
from rich.rule import Rule
from rich.syntax import Syntax

from winter.compiler.compiler import Compiler
from winter.lexer.lexer import Lexer
from winter.nix import render_nix_ast
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
FILE_NIX = THIS_DIR / "draft.nix"
FILE_CONTENTS = FILE.read_text()
LEXED_FORM_TKS, LEXED_ERRORS = Lexer(iter(FILE_CONTENTS)).parse_total()
PARSED_FORM_AST, PARSED_ERRORS = Parser(iter(LEXED_FORM_TKS)).parse_total()
COMPILED_FORM_AST, COMPILE_ERRORS = Compiler(iter(PARSED_FORM_AST)).parse_total()
RENDERED_NIX = [render_nix_ast(x) for x in COMPILED_FORM_AST]


def write_out() -> str:
    _ = FILE_NIX.write_text("\n".join(RENDERED_NIX))
    _ = subprocess.run(
        ["alejandra", str(FILE_NIX.absolute())],
        check=False,
        capture_output=True,
    )
    return FILE_NIX.read_text()


def p(*xs: object) -> None:
    RICH_CL.log(*xs)


def _pp(x: object) -> Pretty:
    return Pretty(
        x,
        indent_size=2,
        margin=10,
    )


p(Rule("FILE CONTENTS"))
p(
    Syntax(
        FILE_CONTENTS,
        lexer="clojure",
    ),
)
# p(Rule("# LEXED FORM", align="left"))
# p(
#     Panel(
#         RICH_HL(" | ".join(x.__repr__() for x in LEXED_FORM_TKS)),
#     ),
#     Panel(
#         _pp(LEXED_ERRORS),
#     ),
# )
#
# p(Rule("PARSED FORM"))
#
# p(
#     Panel(
#         _pp(PARSED_FORM_AST),
#     ),
#     Panel(
#         _pp(PARSED_ERRORS),
#     ),
# )

# p(Rule("COMPILED FORM"))
#
# p(
#     Panel(
#         _pp(COMPILED_FORM_AST),
#     ),
#     Panel(
#         _pp(COMPILE_ERRORS),
#     ),
# )


p(Rule("RENDERED FORM"))

p(
    Syntax(
        write_out(),
        lexer="nix",
    ),
)

from pathlib import Path
from typing import Annotated

from cyclopts import App, Parameter
from cyclopts import validators as v

from winter.handler.loader import SourceHandler
from winter.utils.logging import configure_logging

configure_logging()

app = App(
    name="winter CLI",
    help=(
        "Tooling for compiling Winter code to Nix, as well as "
        "LSP and formatting support in the future."
    ),
)

_dir_validator = v.Path(
    exists=True,
    file_okay=False,
)


@app.command
def compile(
    input_dir: Annotated[Path, Parameter(validator=_dir_validator)],
    output_dir: Annotated[Path, Parameter(validator=_dir_validator)],
):
    handler = SourceHandler(
        input_dir=input_dir,
        output_dir=output_dir,
    )
    handler.handle()

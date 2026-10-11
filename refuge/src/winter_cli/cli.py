import sys
from pathlib import Path
from typing import Annotated, Literal

from cyclopts import App, Parameter
from cyclopts import validators as v
from loguru import logger

from winter.handler.loader import SourceHandler, WinterError
from winter.utils.logging import configure_logging
from winter_ls.server import get_server

configure_logging()

app = App(name="winter CLI", help="CLI for Winter-related tooling")
_dir_validator = v.Path(exists=True, file_okay=False)


@app.command
def compile(
    input_dir: Annotated[Path, Parameter(validator=_dir_validator)],
    output_dir: Annotated[Path, Parameter(validator=_dir_validator)],
) -> None:
    try:
        handler = SourceHandler(
            input_dir=input_dir,
            output_dir=output_dir,
        )
        handler.handle()
    except WinterError as err:
        logger.error("Error during compilation: {}", err)
        sys.exit(1)


@app.command
def langserver(
    mode: Literal["ws", "tcp", "io"] = "io",
    host: str = "127.0.0.1",
    port: int = 28634,
) -> None:
    server = get_server()

    if mode == "io":
        server.start_io()
    elif mode == "tcp":
        server.start_tcp(host, port)
    elif mode == "ws":
        server.start_ws(host, port)

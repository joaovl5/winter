lazy import sys
lazy from pathlib import Path
lazy from typing import Annotated, Literal

lazy from cyclopts import App, Parameter
lazy from cyclopts import validators as v
lazy from loguru import logger

lazy from winter.handler.loader import SourceHandler, WinterError
lazy from winter.utils.logging import configure_logging
lazy from winter_ls.server import get_server

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

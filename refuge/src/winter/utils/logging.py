lazy import sys

lazy from loguru import logger
lazy from rich.console import Console
lazy from rich.logging import RichHandler

cl = Console(
    tab_size=2,
    file=sys.stdout,
)


def configure_logging() -> None:

    logger.remove()
    _ = logger.add(
        RichHandler(
            console=cl,
            rich_tracebacks=True,
            log_time_format="(%H:%M:%S)",
            markup=True,
        ),
        level="TRACE",
        format="{message}",
    )

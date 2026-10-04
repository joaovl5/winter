import subprocess
from collections.abc import Generator, Iterator
from pathlib import Path

from attrs import define
from loguru import logger

from winter import nix
from winter.compiler.compiler import Compiler
from winter.lexer.lexemes import Lexeme
from winter.lexer.lexer import Lexer
from winter.parser.ast import AST
from winter.parser.parser import Parser
from winter.utils.io import read_iter


class WinterError(BaseException): ...


VALID_EXTS = frozenset({".win"})


@define
class SourceHandler:
    input_dir: Path
    output_dir: Path
    valid_exts: frozenset[str] = VALID_EXTS
    format_nix: bool = True

    def handle(self) -> None:
        logger.debug("Starting source loading...")
        # we won't be able to remain this flow of doing it all in one go per file after
        # things such as namespaces get introduced
        for source_file in self._walk_files():
            logger.debug(f"Handling file: {source_file}")
            self._handle_file(source_file)
        logger.debug("Done handling all files")

    def _walk_files(self) -> Generator[Path]:
        for root, _, xs in self.input_dir.walk():
            for x_raw in xs:
                x = root / x_raw
                if not x.is_file() or x.suffix not in self.valid_exts:
                    logger.debug(f"Skipping {x}")
                    continue
                yield x

    def _handle_file(self, file: Path) -> None:
        if not file.exists():
            msg = f"File not found: {file}"
            raise WinterError(msg)

        lexer = self._mk_lexer(read_iter(file))
        lexer_stream = lexer.parse_lazy_strict()
        parser = self._mk_parser(lexer_stream)
        parser_stream = parser.parse_lazy_strict()
        compiler = self._mk_compiler(parser_stream)
        compiler_stream = compiler.parse_lazy_strict()
        compiler_out = list(compiler_stream)
        if len(compiler_out) != 1:
            msg = f"Nix AST didn't compile into one root node for file: {file}"
            raise WinterError(msg)
        compiler_node = compiler_out[0]
        rendered_nix = self._render_nix(compiler_node)

        out_file = self._get_out_counterpart(file)

        _ = out_file.write_text(rendered_nix)

    def _get_out_counterpart(self, input_path: Path) -> Path:
        relative_path = input_path.relative_to(self.input_dir)
        path = self.output_dir / relative_path
        return path.with_suffix(".nix")

    def _mk_lexer(self, data: Iterator[str]) -> Lexer:
        return Lexer(data)

    def _mk_parser(self, data: Iterator[Lexeme.Type]) -> Parser:
        return Parser(data)

    def _mk_compiler(self, data: Iterator[AST.Type]) -> Compiler:
        return Compiler(data)

    def _render_nix(self, data: nix.AST.Type) -> str:
        out = nix.render_nix_ast(data)
        if self.format_nix:
            out = subprocess.run(
                ("alejandra", "-"),
                input=out,
                encoding="utf-8",
                check=False,
                capture_output=True,
            ).stdout
        return out

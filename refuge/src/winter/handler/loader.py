import subprocess
from collections.abc import Generator, Iterator
from pathlib import Path

from attrs import define
from loguru import logger

from winter import nix
from winter.compiler.compiler import Compiler
from winter.lexer.lexemes import Lexeme
from winter.lexer.lexer import Lexer
from winter.parser.ast import AST, Context, Project
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

    def _walk_files(self) -> Generator[Path]:
        for root, _, xs in self.input_dir.walk():
            for x_raw in xs:
                x = root / x_raw
                if not x.is_file() or x.suffix not in self.valid_exts:
                    logger.debug(f"Skipping {x}")
                    continue
                yield x

    def handle(self) -> None:
        logger.debug("Starting source loading...")
        contexts: list[Context] = []
        source_files = tuple(self._walk_files())
        logger.debug("Collected {} files", len(source_files))
        for source_file in source_files:
            logger.debug("Parsing file: {}", source_file)
            # we can try doing a lazy-parse thing later (parse on demand, just-in-time)
            # but for now this eager approach is good enough
            file_ast = self._handle_file(source_file)
            file_ctx = Context(
                file=source_file,
                name=source_file.name,
                body=file_ast,
            )
            contexts.append(file_ctx)
        project = Project(
            dir=self.input_dir,
            name=self.input_dir.name,
            contexts=contexts,
        )
        logger.debug("Starting compilation of {} contexts", len(contexts))
        compiler = self._mk_compiler(project)
        out, err = compiler.compile()
        if out is None or err:
            raise WinterError("Compilation failed", err)
        for ctx_key, nix_ast in out.tree.items():
            input_ctx = compiler._input_tree[ctx_key]
            save_path = self._get_out_counterpart(input_ctx.file)
            rendered_nix = self._render_nix(nix_ast)
            save_path.parent.mkdir(exist_ok=True, parents=True)
            _ = save_path.write_text(rendered_nix)
        logger.debug(
            "Compilation successful, written to {} output files",
            len(out.tree),
        )

    def _handle_file(self, file: Path) -> list[AST.Type]:
        if not file.exists():
            msg = f"File not found: {file}"
            raise WinterError(msg)

        lexer = self._mk_lexer(read_iter(file))
        lexer_stream = lexer.parse_lazy_strict()
        parser = self._mk_parser(lexer_stream)
        parser_stream = parser.parse_lazy_strict()
        return list(parser_stream)

    def _get_out_counterpart(self, input_path: Path) -> Path:
        relative_path = input_path.relative_to(self.input_dir)
        path = self.output_dir / relative_path
        return path.with_suffix(".nix")

    def _mk_lexer(self, data: Iterator[str]) -> Lexer:
        return Lexer(data)

    def _mk_parser(self, data: Iterator[Lexeme.Type]) -> Parser:
        return Parser(data)

    def _mk_compiler(self, data: Project) -> Compiler:
        return Compiler(data)

    def _render_nix(self, data: nix.AST.Type) -> str:
        out = nix.render_nix_ast(data)
        if self.format_nix:
            res = subprocess.run(
                ("alejandra", "-"),
                input=out,
                encoding="utf-8",
                check=False,
                capture_output=True,
            )
            if res.returncode != 0:
                msg = (
                    "formatter `alejandra` raised an error, AST is likely invalid! "
                    f"process stderr: {res.stderr}"
                )
                raise WinterError(msg)
            out = res.stdout
        return out

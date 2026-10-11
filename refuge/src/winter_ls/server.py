from collections.abc import Iterator
from pathlib import Path
from typing import cast

from attrs import define
from loguru import logger
from lsprotocol import types
from more_itertools import collapse
from pygls.lsp.server import LanguageServer
from pygls.workspace import TextDocument

from winter.compiler.compiler import Compiler
from winter.lexer.lexer import Lexer
from winter.parser.ast import Context, Project
from winter.parser.parser import Parser
from winter.utils.ast_utils import convert_position
from winter.utils.voyager import VoyagerError

LANGSERVER_VERSION = "v1"


def _convert_range(a: int, b: int, source: str) -> types.Range:
    ax, ay = convert_position(pos=a, source=source)
    bx, by = convert_position(pos=b, source=source)
    return types.Range(
        start=types.Position(ax, ay),
        end=types.Position(bx, by),
    )


@define
class WinterLanguageServer(LanguageServer):
    def __init__(self, *args: object, **kwargs: object) -> None:
        super().__init__(*args, **kwargs)  # pyright: ignore[reportUnknownMemberType,reportArgumentType]
        self.diagnostics: dict[str, tuple[int | None, list[types.Diagnostic]]] = {}

    def parse(self, document: TextDocument) -> None:
        logger.trace("Parsing URI `{}`", document.uri)
        _, prev = self.diagnostics.get(document.uri, (0, []))
        diagnostics: list[types.Diagnostic] = []
        try:
            lex = Lexer(iter(document.source)).parse_lazy_strict()
            ast = list(Parser(lex).parse_lazy_strict())
            context = Context(file=Path(), name=None, body=ast)
            project = Project(dir=Path(), name=None, contexts=[context])
            out, err = Compiler(project=project).compile()
            if out is None or err:
                raise err or RuntimeError
        except* VoyagerError as exs:
            for ex in cast("Iterator[VoyagerError]", collapse(exs.exceptions)):
                err_sev = types.DiagnosticSeverity.Error
                range_ = _convert_range(ex.pos_start, ex.pos_end, document.source)
                err = types.Diagnostic(message=ex.msg, severity=err_sev, range=range_)
                logger.trace("Adding error diagnostic `{}`", err)
                diagnostics.append(err)
        if prev != diagnostics:
            self.diagnostics[document.uri] = (document.version, diagnostics)


def _ev_did_open(
    ls: WinterLanguageServer,
    params: types.DidOpenTextDocumentParams,
) -> None:
    doc = ls.workspace.get_text_document(params.text_document.uri)
    ls.parse(doc)


def _ev_diagnostic(
    ls: WinterLanguageServer,
    params: types.DocumentDiagnosticParams,
) -> (
    types.FullDocumentDiagnosticReport | types.UnchangedDocumentDiagnosticReport | None
):
    if (uri := params.text_document.uri) not in ls.diagnostics:
        return None
    version, diagnostics = ls.diagnostics[uri]
    result_id = f"{uri}@{version}"
    if result_id == params.previous_result_id:
        return types.UnchangedDocumentDiagnosticReport(result_id)
    return types.FullDocumentDiagnosticReport(
        items=diagnostics,
        result_id=result_id,
    )


def _ev_workspace_diagnostic(
    ls: WinterLanguageServer,
    params: types.WorkspaceDiagnosticParams,
) -> types.WorkspaceDiagnosticReport:
    items: list[
        types.WorkspaceUnchangedDocumentDiagnosticReport
        | types.WorkspaceFullDocumentDiagnosticReport
    ] = []
    prev_ids = {res.value for res in params.previous_result_ids}
    for uri, (version, diagnostics) in ls.diagnostics.items():
        result_id = f"{uri}@{version}"
        if result_id in prev_ids:
            obj = types.WorkspaceUnchangedDocumentDiagnosticReport(
                uri=uri, result_id=result_id, version=version
            )
            items.append(obj)
            continue
        obj = types.WorkspaceFullDocumentDiagnosticReport(
            uri=uri, version=version, items=diagnostics
        )
        items.append(obj)

    return types.WorkspaceDiagnosticReport(items=items)


def get_server() -> WinterLanguageServer:
    server = WinterLanguageServer("winter-language-server", LANGSERVER_VERSION)
    _ = server.feature(types.TEXT_DOCUMENT_DID_OPEN)(_ev_did_open)
    _ = server.feature(types.TEXT_DOCUMENT_DID_CHANGE)(_ev_did_open)
    _ = server.feature(
        types.TEXT_DOCUMENT_DIAGNOSTIC,
        types.DiagnosticOptions(
            identifier="pull-diagnostics",
            inter_file_dependencies=False,
            workspace_diagnostics=True,
        ),
    )(_ev_diagnostic)
    _ = server.feature(types.WORKSPACE_DIAGNOSTIC)(_ev_workspace_diagnostic)
    return server

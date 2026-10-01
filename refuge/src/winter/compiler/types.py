from enum import StrEnum

from winter.parser.ast import CallableHead

_Op = CallableHead.OPERATOR


class OpSpecialCase(StrEnum):
    DOT_SEL = "dot_sel"
    SEL_OR = "sel_or"
    SEL_HAS = "sel_has"


OPERATOR_MAPPING: dict[_Op, str | OpSpecialCase] = {
    _Op.ADD: "+",
    _Op.SUB: "-",
    _Op.MUL: "*",
    _Op.DIV: "/",
    _Op.GT: ">",
    _Op.LT: "<",
    _Op.GTE: ">=",
    _Op.LTE: "<=",
    _Op.EQ: "==",
    _Op.NEQ: "!=",
    _Op.NOT: "!",
    _Op.ALL: "&&",
    _Op.ANY: "||",
    _Op.THUS: "->",
    _Op.SEL: OpSpecialCase.DOT_SEL,
    _Op.OR: OpSpecialCase.SEL_OR,
    _Op.HAS: OpSpecialCase.SEL_HAS,
    _Op.CONC: "++",
    _Op.UPDT: "//",
    _Op.LPIPE: "<|",
    _Op.RPIPE: "|>",
}

UNARY_OPERATORS: set[_Op] = {
    _Op.SUB,
    _Op.NOT,
}

" Vim syntax file
" Language:   Winter
" Maintainer: joaovl5

if exists("b:current_syntax")
    finish
endif

let s:cpo_sav = &cpo
set cpo&vim

if has("folding") && exists("g:winter_fold") && g:winter_fold > 0
    setlocal foldmethod=syntax
endif

syntax keyword WinterCommentTodo contained FIXME XXX TODO FIXME: XXX: TODO:

" Winter comments
syntax match WinterComment "#.*$" contains=WinterCommentTodo,@Spell

syntax match WinterStringEscape '\v\\%([ntvr0zfe"\\]|x[[0-9a-fA-F]]\{2}|u[[0-9a-fA-F]]\{4}|U[[0-9a-fA-F]]\{6})' contained
syntax region WinterString matchgroup=WinterStringDelimiter start=/"/ skip=/\\\\\|\\"/ end=/"/ contains=WinterStringEscape,@Spell
syntax region WinterBuffer matchgroup=WinterStringDelimiter start=/@"/ skip=/\\\\\|\\"/ end=/"/ contains=WinterStringEscape,@Spell
syntax region WinterString matchgroup=WinterStringDelimiter start="\z(`\+\)" end="\z1" contains=@Spell
syntax region WinterBuffer matchgroup=WinterStringDelimiter start="@\z(`\+\)" end="\z1" contains=@Spell

let s:winter_syntax_keywords = {
    \   'WinterBoolean': ["false","true"]
    \ , 'WinterConstant': ["nil"]
    \ , 'WinterSpecialForm': ["if","do","fn","while","def","var","quote","quasiquote","unquote","splice","set","break"]
    \ , 'WinterFunction': ["%","*","*args*","*current-file*","*debug*","*defdyn-prefix*","*doc-color*","*doc-width*","*err*","*err-color*","*executable*","*exit*","*exit-value*","*ffi-context*","*flychecking*","*lint-error*","*lint-levels*","*lint-warn*","*macro-form*","*macro-lints*","*module-cache*","*module-loaders*","*module-loading*","*module-make-env*","*module-paths*","*out*","*peg-grammar*","*pretty-format*","*profilepath*","*redef*","*repl-prompt*","*syspath*","*task-id*","+","-","/","<","<=","=",">",">=","abstract?","accumulate","accumulate2","all","all-bindings","all-dynamics","any?","apply","array","array/clear","array/concat","array/ensure","array/fill","array/insert","array/join","array/new","array/new-filled","array/peek","array/pop","array/push","array/remove","array/slice","array/trim","array/weak","array?","asm","bad-compile","bad-parse","band","blshift","bnot","boolean?","bor","brshift","brushift","buffer","buffer/bit","buffer/bit-clear","buffer/bit-set","buffer/bit-toggle","buffer/blit","buffer/clear","buffer/fill","buffer/format","buffer/format-at","buffer/from-bytes","buffer/new","buffer/new-filled","buffer/popn","buffer/push","buffer/push-at","buffer/push-byte","buffer/push-float32","buffer/push-float64","buffer/push-string","buffer/push-uint16","buffer/push-uint32","buffer/push-uint64","buffer/push-word","buffer/slice","buffer/trim","buffer?","bundle/add","bundle/add-bin","bundle/add-directory","bundle/add-file","bundle/add-manpage","bundle/install","bundle/installed?","bundle/list","bundle/manifest","bundle/prune","bundle/reinstall","bundle/replace","bundle/topolist","bundle/uninstall","bundle/update-all","bundle/whois","bxor","bytes?","cancel","cfunction?","cli-main","cmp","comp","compare","compare<","compare<=","compare=","compare>","compare>=","compile","complement","count","curenv","debug","debug/arg-stack","debug/break","debug/fbreak","debug/lineage","debug/stack","debug/stacktrace","debug/step","debug/unbreak","debug/unfbreak","debugger","debugger-env","debugger-on-status","dec","deep-not=","deep=","default-peg-grammar","defglobal","describe","dictionary?","disasm","distinct","div","doc*","doc-format","doc-of","dofile","drop","drop-until","drop-while","dyn","eflush","empty?","env-lookup","eprin","eprinf","eprint","eprintf","error","errorf","ev/acquire-lock","ev/acquire-rlock","ev/acquire-wlock","ev/all-tasks","ev/call","ev/cancel","ev/capacity","ev/chan","ev/chan-close","ev/chunk","ev/close","ev/count","ev/deadline","ev/full","ev/give","ev/give-supervisor","ev/go","ev/lock","ev/read","ev/release-lock","ev/release-rlock","ev/release-wlock","ev/rselect","ev/rwlock","ev/select","ev/sleep","ev/take","ev/thread","ev/thread-chan","ev/to-file","ev/write","eval","eval-string","even?","every?","extreme","false?","ffi/align","ffi/call","ffi/calling-conventions","ffi/close","ffi/context","ffi/free","ffi/jitfn","ffi/lookup","ffi/malloc","ffi/native","ffi/pointer-buffer","ffi/pointer-cfunction","ffi/read","ffi/signature","ffi/size","ffi/struct","ffi/trampoline","ffi/write","fiber/can-resume?","fiber/current","fiber/getenv","fiber/last-value","fiber/maxstack","fiber/new","fiber/root","fiber/setenv","fiber/setmaxstack","fiber/status","fiber?","file/close","file/flush","file/lines","file/open","file/read","file/seek","file/tell","file/temp","file/write","filewatch/add","filewatch/listen","filewatch/new","filewatch/remove","filewatch/unlisten","filter","find","find-index","first","flatten","flatten-into","flush","flycheck","freeze","frequencies","from-pairs","function?","gccollect","gcinterval","gcsetinterval","gensym","geomean","get","get-in","getline","getproto","group-by","has-key?","has-value?","hash","idempotent?","identity","import*","in","inc","index-of","indexed?","int/s64","int/to-bytes","int/to-number","int/u64","int?","interleave","interpose","invert","winter/build","janet/config-bits","janet/version","juxt*","keep","keep-syntax","keep-syntax!","keys","keyword","keyword/slice","keyword?","kvs","last","length","lengthable?","load-image","load-image-dict","macex","macex1","maclintf","make-env","make-image","make-image-dict","map","mapcat","marshal","math/-inf","math/abs","math/acos","math/acosh","math/asin","math/asinh","math/atan","math/atan2","math/atanh","math/cbrt","math/ceil","math/cos","math/cosh","math/e","math/erf","math/erfc","math/exp","math/exp2","math/expm1","math/floor","math/frexp","math/gamma","math/gcd","math/hypot","math/inf","math/int-max","math/int-min","math/int32-max","math/int32-min","math/lcm","math/ldexp","math/log","math/log-gamma","math/log10","math/log1p","math/log2","math/nan","math/next","math/pi","math/pow","math/random","math/rng","math/rng-buffer","math/rng-int","math/rng-uniform","math/round","math/seedrandom","math/sin","math/sinh","math/sqrt","math/tan","math/tanh","math/trunc","max","max-of","mean","memcmp","merge","merge-into","merge-module","min","min-of","mod","module/add-paths","module/cache","module/expand-path","module/find","module/loaders","module/loading","module/paths","module/value","nan?","nat?","native","neg?","net/accept","net/accept-loop","net/address","net/address-unpack","net/chunk","net/close","net/connect","net/flush","net/listen","net/localname","net/peername","net/read","net/recv-from","net/send-to","net/server","net/setsockopt","net/shutdown","net/socket","net/write","next","nil?","not","not=","number?","odd?","one?","os/arch","os/cd","os/chmod","os/clock","os/compiler","os/cpu-count","os/cryptorand","os/cwd","os/date","os/dir","os/environ","os/execute","os/exit","os/getenv","os/getpid","os/isatty","os/link","os/lstat","os/mkdir","os/mktime","os/open","os/perm-int","os/perm-string","os/pipe","os/posix-chroot","os/posix-exec","os/posix-fork","os/proc-close","os/proc-kill","os/proc-wait","os/readlink","os/realpath","os/rename","os/rm","os/rmdir","os/setenv","os/setlocale","os/shell","os/sigaction","os/sleep","os/spawn","os/stat","os/strftime","os/symlink","os/time","os/touch","os/umask","os/which","pairs","parse","parse-all","parser/byte","parser/clone","parser/consume","parser/eof","parser/error","parser/flush","parser/has-more","parser/insert","parser/new","parser/produce","parser/state","parser/status","parser/where","partial","partition","partition-by","peg/compile","peg/find","peg/find-all","peg/match","peg/replace","peg/replace-all","pos?","postwalk","pp","prewalk","prin","prinf","print","printf","product","propagate","put","put-in","quit","range","reduce","reduce2","repl","require","resume","return","reverse","reverse!","root-env","run-context","sandbox","scan-number","setdyn","signal","slice","slurp","some","sort","sort-by","sorted","sorted-by","spit","stderr","stdin","stdout","string","string/ascii-lower","string/ascii-upper","string/bytes","string/check-set","string/find","string/find-all","string/format","string/from-bytes","string/has-prefix?","string/has-suffix?","string/join","string/repeat","string/replace","string/replace-all","string/reverse","string/slice","string/split","string/trim","string/triml","string/trimr","string?","struct","struct/getproto","struct/proto-flatten","struct/rawget","struct/to-table","struct/with-proto","struct?","sum","symbol","symbol/slice","symbol?","table","table/clear","table/clone","table/getproto","table/new","table/proto-flatten","table/rawget","table/setproto","table/to-struct","table/weak","table/weak-keys","table/weak-values","table?","take","take-until","take-while","thaw","thaw-keep-keys","trace","true?","truthy?","tuple","tuple/brackets","tuple/join","tuple/setmap","tuple/slice","tuple/sourcemap","tuple/type","tuple?","type","unmarshal","untrace","update","update-in","values","varglobal","walk","warn-compile","xprin","xprinf","xprint","xprintf","yield","zero?","zipcoll"]
    \ , 'WinterMacro': ["%=","*=","++","+=","--","-=","->","->>","-?>","-?>>","/=","and","as->","as-macro","as?->","assert","assertf","case","catseq","chr","comment","compif","comptime","compwhen","cond","coro","def-","default","defdyn","defer","defmacro","defmacro-","defn","defn-","delay","doc","each","eachk","eachp","edefer","ev/do-thread","ev/gather","ev/spawn","ev/spawn-thread","ev/with-deadline","ev/with-lock","ev/with-rlock","ev/with-wlock","ffi/defbind","ffi/defbind-alias","fiber-fn","for","forever","forv","generate","if-let","if-not","if-with","import","juxt","label","let","loop","match","or","prompt","protect","repeat","seq","short-fn","tabseq","toggle","tracev","try","unless","use","var-","varfn","when","when-let","when-with","with","with-dyns","with-env","with-syms","with-vars"]
    \ }

function! s:syntax_keyword(dict)
    for key in keys(a:dict)
        execute 'syntax keyword' key join(a:dict[key], ' ')
    endfor
endfunction

call s:syntax_keyword(s:winter_syntax_keywords)

unlet! s:key
delfunction s:syntax_keyword

" Try symchars but handle old vim versions.
try
    let s:symcharnodig = '\!\$%\&\*\+\-./:<=>?@A-Z^_a-z|\x80-\U10FFFF'
    " Make sure we support large character ranges in this vim version.
    execute 'syntax match WinterSymbolDud "\v<%([' . s:symcharnodig . '])%([' . s:symchar . '])*>"'
catch
    let s:symcharnodig = '\!\$%\&\*\+\-./:<=>?@A-Z^_a-z'
endtry

" Winter special characters
syntax match WinterQuote "'"
syntax match WinterSplice ";"
syntax match WinterQuasiquote "\~"
syntax match WinterUnquote ","
syntax match WinterShortFn "|"

" Winter symbols
let s:symchar = '0-9' . s:symcharnodig
execute 'syntax match WinterSymbol "\v<%([' . s:symcharnodig . '])%([' . s:symchar . '])*>"'
execute 'syntax match WinterKeyword "\v<:%([' . s:symchar . '])*>"'
execute 'syntax match WinterQuoted "\v' . "'" . '<%([' . s:symcharnodig . '])%([' . s:symchar . '])*>"'
unlet! s:symchar s:symcharnodig

" Winter numbers
function! s:syntaxNumber(prefix, expo, digit)
  let l:digit = '[_' . a:digit . ']'
  execute 'syntax match WinterNumber "\v\c<[-+]?' . a:prefix . '%(' .
              \ l:digit . '+|' .
              \ l:digit . '+\.' . l:digit . '*|' .
              \ '\.' . l:digit . '+)%(' . a:expo . '[-+]?[' . a:digit . ']+)?>"'
endfunction
let s:radix_chars = "0123456789abcdefghijklmnopqrstuvwxyz"
for s:radix in range(2, 36)
    call s:syntaxNumber(s:radix . 'r', '\&', '[' . strpart(s:radix_chars, 0, s:radix) . ']')
endfor
call s:syntaxNumber('', '[&e]', '0123456789')
call s:syntaxNumber('0x', '\&', '0123456789abcdef')
unlet! s:radix_chars s:radix

" -*- TOP CLUSTER -*-
syntax cluster WinterTop contains=@Spell,WinterComment,WinterConstant,WinterQuoted,WinterKeyword,WinterSymbol,WinterNumber,WinterString,WinterBuffer,WinterTuple,WinterArray,WinterTable,WinterStruct,WinterSpecialForm,WinterBoolean,WinterFunction,WinterMacro

syntax region WinterTuple matchgroup=WinterParen start="("  end=")" contains=@WinterTop fold
syntax region WinterArray matchgroup=WinterParen start="@("  end=")" contains=@WinterTop fold
syntax region WinterTuple matchgroup=WinterParen start="\[" end="]" contains=@WinterTop fold
syntax region WinterArray matchgroup=WinterParen start="@\[" end="]" contains=@WinterTop fold
syntax region WinterTable matchgroup=WinterParen start="{"  end="}" contains=@WinterTop fold
syntax region WinterStruct matchgroup=WinterParen start="@{"  end="}" contains=@WinterTop fold

" Highlight superfluous closing parens, brackets and braces.
syntax match WinterError "]\|}\|)"

syntax sync fromstart

" Highlighting
hi def link WinterComment Comment
hi def link WinterSymbol Normal
hi def link WinterQuoted Identifier
hi def link WinterNumber Number
hi def link WinterConstant Constant
hi def link WinterBoolean Boolean
hi def link WinterKeyword Keyword
hi def link WinterSpecialForm Special
hi def link WinterFunction Function
hi def link WinterMacro Macro
hi def link WinterBuffer String
hi def link WinterString String
hi def link WinterStringDelimiter String
hi def link WinterStringEscape Character
hi def link WinterQuote SpecialChar
hi def link WinterSplice SpecialChar
hi def link WinterQuasiquote SpecialChar
hi def link WinterUnquote SpecialChar
hi def link WinterShortFn SpecialChar
hi def link WinterParen Delimiter
hi def link WinterError Error

let b:current_syntax = "winter"

let &cpo = s:cpo_sav
unlet! s:cpo_sav

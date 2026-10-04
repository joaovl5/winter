" Vim filetype plugin file
" Language: WINTER
" Maintainer: joaovl5

if exists("b:did_ftplugin")
  finish
endif

let b:did_ftplugin = 1

setlocal iskeyword=!,$,%,#,*,+,-,.,/,:,<,=,>,?,_,a-z,A-Z,48-57,128-247,124,126,38,94

setlocal define=\\v[(/]def(ault)@!\\S*

setlocal suffixesadd=.win

" avoid autowrapping code
setlocal formatoptions-=t

setlocal comments=n:;
setlocal commentstring=;\ %s


" tiny hack to use parinfer-rust without upstream having to support it, given that the
" language's syntax is pretty much like clojure's for this purpose
if exists('#Parinfer#FileType#clojure')
  doautocmd <nomodeline> Parinfer FileType clojure
endif

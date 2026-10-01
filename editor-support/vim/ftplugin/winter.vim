" Vim filetype plugin file
" Language: WINTER
" Maintainer: joaovl5

if exists("b:did_ftplugin")
  finish
endif

let b:did_ftplugin = 1

let s:cpo_save = &cpo

setlocal iskeyword=!,$,%,#,*,+,-,.,/,:,<,=,>,?,_,a-z,A-Z,48-57,128-247,124,126,38,94

setlocal define=\\v[(/]def(ault)@!\\S*

setlocal suffixesadd=.fnl

" avoid autowrapping code
setlocal formatoptions-=t

setlocal comments=n:;
setlocal commentstring=;\ %s

let &cpo = s:cpo_save


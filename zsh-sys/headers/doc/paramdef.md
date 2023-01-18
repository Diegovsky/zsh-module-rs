# Zsh global variable definition
zsh allows you to define what it calls "params" which AFAIK are global values.

The example module (zsh/example) defines "exarr", "exint", and "exstr" parameters, respectively an array, int and string.

They are printed when you call the command `example`. Try changing the value of `exstr` and calling `example`.
```zsh
exstr=wowzers

example
```

output:
```
Options: 
Arguments:
Name: example

Integer Parameter: 0
String Parameter: wowzers
Array Parameter:
```
